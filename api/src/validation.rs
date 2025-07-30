use crate::db;
use crate::domain::SearchResponse;
use crate::errors::PgError;
use deadpool_postgres::Client;
use log::{info, warn};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{PointId, QueryPointsBuilder, RecommendInputBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Concept {
    #[serde(rename = "CONCEPT_ID")]
    pub concept_id: i32,
    #[serde(rename = "CONCEPT_NAME")]
    pub concept_name: String,
    #[serde(rename = "VOCABULARY_ID")]
    pub vocabulary_id: String,
    #[serde(rename = "DOMAIN_ID")]
    pub domain_id: String,
    #[serde(rename = "CONCEPT_CLASS_ID")]
    pub concept_class_id: String,
    #[serde(rename = "STANDARD_CONCEPT")]
    pub standard_concept: Option<String>,
    #[serde(rename = "STANDARD_CONCEPT_CAPTION")]
    pub standard_concept_caption: Option<String>,
    #[serde(rename = "INVALID_REASON")]
    pub invalid_reason: Option<String>,
    #[serde(rename = "INVALID_REASON_CAPTION")]
    pub invalid_reason_caption: Option<String>,
    #[serde(rename = "CONCEPT_CODE")]
    pub concept_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConceptSetItem {
    pub concept: Concept,
    #[serde(rename = "isExcluded")]
    pub is_excluded: bool,
    #[serde(rename = "includeDescendants")]
    pub include_descendants: bool,
    #[serde(rename = "includeMapped")]
    pub include_mapped: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConceptSetExpression {
    pub items: Vec<ConceptSetItem>,
}

#[derive(Debug, Deserialize)]
pub struct ConceptSetWithMetadata {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub expression: ConceptSetExpression,
}

#[derive(Debug)]
pub struct ConceptGatheringResult {
    pub included_concepts: Vec<i32>,
    pub included_descendants: Vec<i32>, // Will store actual descendant concept IDs
    pub included_mapped: Vec<i32>,
    pub excluded_concepts: Vec<i32>,
    pub excluded_descendants: Vec<i32>, // Will store actual descendant concept IDs
    pub excluded_mapped: Vec<i32>,
}

impl ConceptGatheringResult {
    pub fn new() -> Self {
        Self {
            included_concepts: Vec::new(),
            included_descendants: Vec::new(),
            included_mapped: Vec::new(),
            excluded_concepts: Vec::new(),
            excluded_descendants: Vec::new(),
            excluded_mapped: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub concept_summary: Option<ConceptGatheringResult>,
    pub recommendations: Option<ConceptRecommendations>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            concept_summary: None,
            recommendations: None,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn to_json(&self) -> Value {
        let mut result = serde_json::json!({
            "valid": self.valid,
            "errors": self.errors,
            "warnings": self.warnings
        });

        if let Some(summary) = &self.concept_summary {
            result["concept_summary"] = serde_json::json!({
                "included_concepts_count": summary.included_concepts.len(),
                "included_descendants_count": summary.included_descendants.len(),
                "included_mapped_count": summary.included_mapped.len(),
                "excluded_concepts_count": summary.excluded_concepts.len(),
                "excluded_descendants_count": summary.excluded_descendants.len(),
                "excluded_mapped_count": summary.excluded_mapped.len(),
                "total_included": summary.included_concepts.len() + summary.included_descendants.len() + summary.included_mapped.len(),
                "total_excluded": summary.excluded_concepts.len() + summary.excluded_descendants.len() + summary.excluded_mapped.len()
            });
        }

        if let Some(recommendations) = &self.recommendations {
            result["recommendations"] =
                serde_json::to_value(recommendations).unwrap_or(serde_json::json!(null));
        }

        result
    }
}

fn parse_concept_set(concept_set: &str) -> Result<ConceptSetExpression, String> {
    // Try to parse as direct expression format first
    if let Ok(expression) = serde_json::from_str::<ConceptSetExpression>(concept_set) {
        return Ok(expression);
    }

    // Try to parse as concept set with metadata format
    if let Ok(concept_set_with_metadata) =
        serde_json::from_str::<ConceptSetWithMetadata>(concept_set)
    {
        return Ok(concept_set_with_metadata.expression);
    }

    Err("Unable to parse concept set".to_string())
}

fn gather_concepts_from_expression(expression: &ConceptSetExpression) -> ConceptGatheringResult {
    let mut result = ConceptGatheringResult::new();

    for item in &expression.items {
        let concept_id = item.concept.concept_id;

        if item.is_excluded {
            // Add to excluded lists - concept is always excluded
            result.excluded_concepts.push(concept_id);

            // Note: We'll populate actual descendants and mapped concepts later in the validation function
            // For now, just collect the direct concepts

            // Track mapped concepts (will be populated later with actual mapped concept IDs)
            if item.include_mapped {
                result.excluded_mapped.push(concept_id);
            }
        } else {
            // Add to included lists - concept is always included
            result.included_concepts.push(concept_id);

            // Note: We'll populate actual descendants and mapped concepts later in the validation function
            // For now, just collect the direct concepts

            // Track mapped concepts (will be populated later with actual mapped concept IDs)
            if item.include_mapped {
                result.included_mapped.push(concept_id);
            }
        }
    }

    info!(
        "Gathered direct concepts - Included: {}, Excluded: {}",
        result.included_concepts.len(),
        result.excluded_concepts.len()
    );

    result
}

fn sort_and_dedup_vec(vec: &mut Vec<i32>) {
    vec.sort();
    vec.dedup();
}

pub async fn analyze_concept_set(
    concept_set: &str,
    pg_client: &Client,
    qdrant_client: Option<&Qdrant>,
    concept_index: Option<&HashMap<String, Vec<Uuid>>>,
) -> Result<ValidationResult, PgError> {
    info!("Starting concept set analysis");
    let mut result = ValidationResult::new();

    // Basic validation checks
    if concept_set.trim().is_empty() {
        result.add_error("Concept set cannot be empty".to_string());
        return Ok(result);
    }

    // Try to parse the JSON in either format
    let expression = match parse_concept_set(concept_set) {
        Ok(expr) => expr,
        Err(e) => {
            result.add_error(format!("Invalid concept set format: {}", e));
            return Ok(result);
        }
    };

    // Validate structure
    if expression.items.is_empty() {
        result.add_error("Concept set expression contains no items".to_string());
        return Ok(result);
    }

    // Gather concepts from the expression
    let mut concept_summary = gather_concepts_from_expression(&expression);

    // Basic logical validation
    if concept_summary.included_concepts.is_empty()
        && concept_summary.included_descendants.is_empty()
        && concept_summary.included_mapped.is_empty()
    {
        result.add_warning("No concepts are included in this concept set".to_string());
    }

    check_for_duplicates(&mut result, &expression);

    // Collect all concept IDs that need descendant expansion
    let concepts_needing_descendants: Vec<i32> = expression
        .items
        .iter()
        .filter(|item| item.include_descendants)
        .map(|item| item.concept.concept_id)
        .collect();

    // Batch fetch all descendants if needed
    if !concepts_needing_descendants.is_empty() {
        match db::get_batch_descendant_concepts(pg_client, &concepts_needing_descendants).await {
            Ok(descendants_map) => {
                // Process each item and add descendants to appropriate lists
                for item in &expression.items {
                    let concept_id = item.concept.concept_id;

                    if item.include_descendants {
                        if let Some(descendants) = descendants_map.get(&concept_id) {
                            info!(
                                "Found {} descendants for concept {}",
                                descendants.len(),
                                concept_id
                            );

                            if item.is_excluded {
                                // Add descendants to excluded list
                                concept_summary.excluded_descendants.extend(descendants);
                            } else {
                                // Add descendants to included list
                                concept_summary.included_descendants.extend(descendants);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                result.add_warning(format!("Could not get descendants for concepts: {}", e));
            }
        }
    }

    // Collect all concept IDs that need mapped expansion
    let concepts_needing_mapped: Vec<i32> = expression
        .items
        .iter()
        .filter(|item| item.include_mapped)
        .map(|item| item.concept.concept_id)
        .collect();

    // Batch fetch all mapped concepts if needed
    if !concepts_needing_mapped.is_empty() {
        match db::get_batch_mapped_concepts(pg_client, &concepts_needing_mapped).await {
            Ok(mapped_map) => {
                // Process each item and add mapped concepts to appropriate lists
                for item in &expression.items {
                    let concept_id = item.concept.concept_id;

                    if item.include_mapped {
                        if let Some(mapped) = mapped_map.get(&concept_id) {
                            info!(
                                "Found {} mapped concepts for concept {}",
                                mapped.len(),
                                concept_id
                            );

                            if item.is_excluded {
                                // Add mapped concepts to excluded list
                                concept_summary.excluded_mapped.extend(mapped);
                            } else {
                                // Add mapped concepts to included list
                                concept_summary.included_mapped.extend(mapped);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                result.add_warning(format!("Could not get mapped concepts for concepts: {}", e));
            }
        }
    }

    // Remove duplicates from descendant and mapped lists
    sort_and_dedup_vec(&mut concept_summary.included_descendants);
    sort_and_dedup_vec(&mut concept_summary.excluded_descendants);
    sort_and_dedup_vec(&mut concept_summary.included_mapped);
    sort_and_dedup_vec(&mut concept_summary.excluded_mapped);

    // Create sets of all excluded concepts (direct + descendants + mapped) for efficient lookup
    let mut all_excluded: HashSet<i32> = HashSet::new();
    all_excluded.extend(&concept_summary.excluded_concepts);
    all_excluded.extend(&concept_summary.excluded_descendants);
    all_excluded.extend(&concept_summary.excluded_mapped);

    // Remove excluded concepts from included lists
    concept_summary
        .included_concepts
        .retain(|&concept_id| !all_excluded.contains(&concept_id));
    concept_summary
        .included_descendants
        .retain(|&concept_id| !all_excluded.contains(&concept_id));
    concept_summary
        .included_mapped
        .retain(|&concept_id| !all_excluded.contains(&concept_id));

    info!(
        "Final counts - Included concepts: {}, Included descendants: {}, Included mapped: {}, Excluded concepts: {}, Excluded descendants: {}, Excluded mapped: {}",
        concept_summary.included_concepts.len(),
        concept_summary.included_descendants.len(),
        concept_summary.included_mapped.len(),
        concept_summary.excluded_concepts.len(),
        concept_summary.excluded_descendants.len(),
        concept_summary.excluded_mapped.len()
    );

    result.concept_summary = Some(concept_summary);

    // Generate recommendations if qdrant client and concept index are available
    if let (Some(qdrant), Some(index)) = (qdrant_client, concept_index) {
        match get_concept_recommendations(&expression, pg_client, qdrant, index, 50).await {
            Ok(recommendations) => {
                result.recommendations = Some(recommendations);
            }
            Err(e) => {
                result.add_warning(format!("Could not generate recommendations: {}", e));
            }
        }
    }

    // TODO: Add more database validation
    // - Verify concept IDs exist in the vocabulary
    // - Check for invalid standard_concept values
    // - Validate vocabulary_id, domain_id, concept_class_id
    // - Get mapped concepts using concept_relationship table

    info!("Concept set analysis completed");
    Ok(result)
}

fn check_for_duplicates(result: &mut ValidationResult, expression: &ConceptSetExpression) {
    // Check for duplicate concept IDs within the same expression
    let all_concept_ids: Vec<i32> = expression
        .items
        .iter()
        .map(|item| item.concept.concept_id)
        .collect();
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();

    for &concept_id in &all_concept_ids {
        if !seen.insert(concept_id) {
            duplicates.insert(concept_id);
        }
    }

    if !duplicates.is_empty() {
        let mut duplicate_list: Vec<i32> = duplicates.into_iter().collect();
        duplicate_list.sort();
        result.add_warning(format!(
            "Duplicate concept IDs found in expression: {}",
            duplicate_list
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
}

#[derive(Debug, Serialize)]
pub struct RecommendedConcept {
    pub concept_id: i32,
    pub concept_name: String,
    pub vocabulary_id: String,
    pub domain_id: String,
    pub concept_class_id: String,
    pub concept_code: String,
    pub standard_concept: String,
    pub invalid_reason: Option<String>,
    pub similarity_score: f32,
    pub source_concept_id: i32, // The top-level concept that led to this recommendation
}

#[derive(Debug, Serialize)]
pub struct ConceptRecommendations {
    pub recommendations: Vec<RecommendedConcept>,
    pub total_count: usize,
    pub used_vocabularies: Vec<String>,
}

fn process_concepts_from_cache(
    concepts: &[&ConceptSetItem],
    concept_index: &HashMap<String, Vec<Uuid>>,
    mut source_concept_map: Option<&mut HashMap<String, i32>>,
    log_prefix: &str,
) -> Vec<PointId> {
    let mut point_ids = Vec::new();

    for item in concepts {
        let concept_name = &item.concept.concept_name;
        let concept_name_lower = concept_name.to_lowercase();
        let concept_id = item.concept.concept_id;

        info!(
            "{} for concept: {} (ID: {})",
            log_prefix, concept_name, concept_id
        );

        if let Some(cached_ids) = concept_index.get(concept_name_lower.as_str()) {
            // For recommendations, use only the first cached vector per concept to avoid redundancy
            if let Some(first_uuid) = cached_ids.first() {
                let point_id = PointId::from(first_uuid.to_string().as_str());
                point_ids.push(point_id.clone());

                if let Some(ref mut map) = source_concept_map {
                    map.insert(first_uuid.to_string(), concept_id);
                }
            }
        } else {
            warn!(
                "Concept '{}' (lowercase: '{}') not found in cache",
                concept_name, concept_name_lower
            );
        }
    }

    point_ids
}

fn limit_point_ids(point_ids: Vec<PointId>, limit: usize, collection_type: &str) -> Vec<PointId> {
    let original_count = point_ids.len();
    let limited: Vec<_> = point_ids.into_iter().take(limit).collect();

    if limited.len() < original_count {
        info!(
            "Limited {} examples from {} to {} for performance",
            collection_type,
            original_count,
            limited.len()
        );
    }

    limited
}

fn collect_positive_point_ids(
    top_level_included: &[&ConceptSetItem],
    concept_index: &HashMap<String, Vec<Uuid>>,
    source_concept_map: &mut HashMap<String, i32>,
) -> Vec<PointId> {
    process_concepts_from_cache(
        top_level_included,
        concept_index,
        Some(source_concept_map),
        "Getting positive recommendations",
    )
}

fn collect_negative_point_ids(
    expression: &ConceptSetExpression,
    concept_index: &HashMap<String, Vec<Uuid>>,
) -> Vec<PointId> {
    let excluded_concepts: Vec<&ConceptSetItem> = expression
        .items
        .iter()
        .filter(|item| item.is_excluded)
        .collect();

    process_concepts_from_cache(
        &excluded_concepts,
        concept_index,
        None,
        "Getting negative examples",
    )
}

async fn query_and_process_recommendations(
    qdrant_client: &Qdrant,
    recommend_query: qdrant_client::qdrant::Query,
    existing_concepts: &HashSet<i32>,
    top_level_included: &[&ConceptSetItem],
    allowed_domains: &HashSet<String>,
    concept_set_vocabularies: HashSet<String>,
    limit_per_concept: u64,
) -> ConceptRecommendations {
    const COLLECTION_NAME: &str = "meddra";
    let mut all_recommendations = Vec::new();

    let query_points_builder = QueryPointsBuilder::new(COLLECTION_NAME)
        .with_payload(true)
        .score_threshold(0.50)
        .limit(500)
        .query(recommend_query);

    match qdrant_client.query(query_points_builder).await {
        Ok(query_result) => {
            info!(
                "Qdrant query returned {} results",
                query_result.result.len()
            );
            let mut passed_filters_count = 0;
            let mut already_in_set_count = 0;
            let mut wrong_domain_count = 0;

            for scored_point in query_result.result {
                // Use the same approach as the search endpoint
                let search_response = SearchResponse::from(scored_point.clone());

                // Process each concept in the concepts array
                for concept in search_response.concepts {
                    let concept_id = concept.concept_id;

                    // Filter: only not already in set and in allowed domains (let UI handle standard/vocabulary filtering)
                    if !existing_concepts.contains(&concept_id)
                        && allowed_domains.contains(&concept.domain_id)
                    {
                        passed_filters_count += 1;
                        // Use first source concept as default (could be improved to track actual source)
                        let source_concept_id = top_level_included
                            .first()
                            .map(|item| item.concept.concept_id)
                            .unwrap_or(0);

                        all_recommendations.push(RecommendedConcept {
                            concept_id,
                            concept_name: concept.concept_name,
                            vocabulary_id: concept.vocabulary_id,
                            domain_id: concept.domain_id,
                            concept_class_id: concept.concept_class_id,
                            concept_code: concept.concept_code,
                            standard_concept: concept
                                .standard_concept
                                .unwrap_or_else(|| "".to_string()),
                            invalid_reason: concept.invalid_reason,
                            similarity_score: scored_point.score,
                            source_concept_id,
                        });
                    } else if existing_concepts.contains(&concept_id) {
                        already_in_set_count += 1;
                    } else if !allowed_domains.contains(&concept.domain_id) {
                        wrong_domain_count += 1;
                    }
                }
            }

            info!(
                "Filtering results: {} passed filters, {} already in set, {} wrong domain",
                passed_filters_count, already_in_set_count, wrong_domain_count
            );
        }
        Err(e) => {
            info!("Error getting recommendations from Qdrant: {}", e);
        }
    }

    // Sort by similarity score (descending) and limit results
    all_recommendations
        .sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
    let total_count = all_recommendations.len();

    // Get vocabularies from the original concept set (not from recommendations)
    let used_vocabularies: Vec<String> = concept_set_vocabularies.into_iter().collect();

    ConceptRecommendations {
        recommendations: all_recommendations,
        total_count,
        used_vocabularies,
    }
}

pub async fn get_concept_recommendations(
    expression: &ConceptSetExpression,
    pg_client: &Client,
    qdrant_client: &Qdrant,
    concept_index: &HashMap<String, Vec<Uuid>>,
    limit_per_concept: u64,
) -> Result<ConceptRecommendations, PgError> {
    // Get all concepts that are already in the set (direct, descendants, excluded)
    let existing_concepts = get_all_concepts_in_set(expression, pg_client).await?;
    info!(
        "Found {} existing concepts in set to exclude from recommendations",
        existing_concepts.len()
    );

    // Get top-level included concepts
    let top_level_included: Vec<&ConceptSetItem> = expression
        .items
        .iter()
        .filter(|item| !item.is_excluded)
        .collect();

    info!(
        "Found {} top-level included concepts for recommendations",
        top_level_included.len()
    );

    // Collect allowed domain IDs from all concepts in the expression
    let allowed_domains: HashSet<String> = expression
        .items
        .iter()
        .map(|item| item.concept.domain_id.clone())
        .collect();

    // Collect vocabulary IDs from the concept set (for UI pre-selection, not filtering)
    let concept_set_vocabularies: HashSet<String> = expression
        .items
        .iter()
        .map(|item| item.concept.vocabulary_id.clone())
        .collect();

    info!("Allowed domains for recommendations: {:?}", allowed_domains);
    info!(
        "Concept set vocabularies for UI pre-selection: {:?}",
        concept_set_vocabularies
    );

    let mut source_concept_map: HashMap<String, i32> = HashMap::new();

    // Collect positive and negative point IDs
    let all_positive_point_ids =
        collect_positive_point_ids(&top_level_included, concept_index, &mut source_concept_map);
    let all_negative_point_ids = collect_negative_point_ids(expression, concept_index);

    if all_positive_point_ids.is_empty() {
        return Ok(ConceptRecommendations {
            recommendations: Vec::new(),
            total_count: 0,
            used_vocabularies: Vec::new(),
        });
    }

    // Limit to 50 points for performance (Qdrant performance scales linearly with number of examples)
    let limited_positive_point_ids = limit_point_ids(all_positive_point_ids, 50, "positive");
    let limited_negative_point_ids = limit_point_ids(all_negative_point_ids, 50, "negative");

    // Use Qdrant's recommendation API with the cached point IDs
    let mut recs = RecommendInputBuilder::default();
    for point_id in &limited_positive_point_ids {
        recs = recs.add_positive(point_id.clone());
    }
    for point_id in &limited_negative_point_ids {
        recs = recs.add_negative(point_id.clone());
    }

    // Query Qdrant and process results
    let all_recommendations = query_and_process_recommendations(
        qdrant_client,
        recs.build().into(),
        &existing_concepts,
        &top_level_included,
        &allowed_domains,
        concept_set_vocabularies,
        limit_per_concept,
    )
    .await;

    Ok(all_recommendations)
}

async fn get_all_concepts_in_set(
    expression: &ConceptSetExpression,
    pg_client: &Client,
) -> Result<HashSet<i32>, PgError> {
    let mut all_concepts = HashSet::new();

    // Add all direct concepts (included and excluded)
    for item in &expression.items {
        all_concepts.insert(item.concept.concept_id);
    }

    // Collect all concept IDs that need descendant expansion
    let concepts_needing_descendants: Vec<i32> = expression
        .items
        .iter()
        .filter(|item| item.include_descendants)
        .map(|item| item.concept.concept_id)
        .collect();

    // Batch fetch all descendants if needed
    if !concepts_needing_descendants.is_empty() {
        match db::get_batch_descendant_concepts(pg_client, &concepts_needing_descendants).await {
            Ok(descendants_map) => {
                // Add all descendants to the set
                for descendants in descendants_map.values() {
                    all_concepts.extend(descendants);
                }
            }
            Err(_) => {
                // Skip if we can't get descendants
            }
        }
    }

    // Collect all concept IDs that need mapped expansion
    let concepts_needing_mapped: Vec<i32> = expression
        .items
        .iter()
        .filter(|item| item.include_mapped)
        .map(|item| item.concept.concept_id)
        .collect();

    // Batch fetch all mapped concepts if needed
    if !concepts_needing_mapped.is_empty() {
        match db::get_batch_mapped_concepts(pg_client, &concepts_needing_mapped).await {
            Ok(mapped_map) => {
                // Add all mapped concepts to the set
                for mapped in mapped_map.values() {
                    all_concepts.extend(mapped);
                }
            }
            Err(_) => {
                // Skip if we can't get mapped concepts
            }
        }
    }

    Ok(all_concepts)
}
