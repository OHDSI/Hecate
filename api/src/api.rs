use crate::concept_graph::ConceptExpander;
use crate::domain::{Concept, PhoebeResponse, SearchResponse};
use crate::embeddings::fetch_embeddings_cached;
use crate::errors::PgError;
use crate::umls::get_umls_definition_from_nlm;
use crate::utils::deserialize_string_or_vec;
use crate::validation;
use crate::{StateWrapper, db};
use actix_web::web::{Data, Json, Query};
use actix_web::{Error, HttpResponse, error::ErrorInternalServerError, get, post, web};
use log::info;
use moka::future::Cache;
use qdrant_client::qdrant::condition::ConditionOneOf;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::{
    Condition, Filter, GetPointsBuilder, PointId, RetrievedPoint, ScoredPoint, ScrollPointsBuilder,
    SearchPointsBuilder,
};
use qdrant_client::{Qdrant, qdrant};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap, HashSet};

pub const CONCEPT_COLLECTION: &str = "meddra";
pub const SYNONYMS_COLLECTION: &str = "synonyms";

fn normalized_values(values: &[String]) -> Vec<String> {
    let mut normalized = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    normalized.sort_unstable();
    normalized
}

fn search_cache_key(params: &Parameters, default_limit: u64) -> String {
    let mut parts = vec![
        format!("q={}", params.q.trim()),
        format!("limit={}", params.limit.unwrap_or(default_limit)),
    ];
    if let Some(vocab_ids) = &params.vocabulary_id {
        let sorted = normalized_values(vocab_ids);
        parts.push(format!("vocab={}", sorted.join(",")));
    }
    if let Some(exclude_vocab_ids) = &params.exclude_vocabulary_id {
        let sorted = normalized_values(exclude_vocab_ids);
        parts.push(format!("excl_vocab={}", sorted.join(",")));
    }
    if let Some(sc) = &params.standard_concept {
        parts.push(format!("std={}", sc.trim()));
    }
    if let Some(domain_ids) = &params.domain_id {
        let sorted = normalized_values(domain_ids);
        parts.push(format!("domain={}", sorted.join(",")));
    }
    if let Some(class_ids) = &params.concept_class_id {
        let sorted = normalized_values(class_ids);
        parts.push(format!("class={}", sorted.join(",")));
    }
    parts.join("&")
}

#[derive(Clone, Debug, Deserialize)]
struct Parameters {
    q: String,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    vocabulary_id: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    exclude_vocabulary_id: Option<Vec<String>>,
    standard_concept: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    domain_id: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    concept_class_id: Option<Vec<String>>,
    limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ConceptSetValidationRequest {
    concept_set: String,
}

#[derive(Debug, Deserialize)]
struct ExpandParams {
    childlevels: Option<i32>,
    parentlevels: Option<i32>,
}

/// Preserve insertion order and the first result's score while indexing current names.
/// Several groups can acquire the same name when appending a standard concept.
#[derive(Default)]
struct SearchResultGroups {
    results: Vec<SearchResponse>,
    positions: HashMap<String, BTreeSet<usize>>,
}

impl SearchResultGroups {
    fn push(&mut self, mut incoming: SearchResponse) {
        if let Some(positions) = self.positions.remove(&incoming.concept_name_lower) {
            // Visit matching groups in insertion order, just like the original scan.
            // append_concepts drains the incoming concepts into the first match.
            for position in positions {
                let existing = &mut self.results[position];
                existing.append_concepts(&mut incoming.concepts);
                self.positions
                    .entry(existing.concept_name_lower.clone())
                    .or_default()
                    .insert(position);
            }
        } else {
            self.positions
                .entry(incoming.concept_name_lower.clone())
                .or_default()
                .insert(self.results.len());
            self.results.push(incoming);
        }
    }

    fn into_results(self) -> Vec<SearchResponse> {
        self.results
    }
}

fn non_standard_source_ids(main: &[SearchResponse], synonyms: &[SearchResponse]) -> Vec<i32> {
    main.iter()
        .chain(synonyms)
        .flat_map(|result| &result.concepts)
        .filter(|concept| {
            !concept
                .standard_concept
                .as_deref()
                .is_some_and(|sc| sc.eq_ignore_ascii_case("S"))
        })
        .map(|concept| concept.concept_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn process_search_results(
    search_results: Vec<SearchResponse>,
    concept_map: &mut HashMap<i32, (Concept, f64)>,
    parameters: &Parameters,
    record_counts: &HashMap<i32, i64>,
    standard_mappings: &HashMap<i32, Vec<Concept>>,
    limit: usize,
) {
    // First pass: collect all concepts with their best scores
    let mut all_concepts: HashMap<i32, (Concept, f64)> = HashMap::new();

    for sr in search_results {
        for concept in &sr.concepts {
            if concept
                .standard_concept
                .as_ref()
                .is_some_and(|sc| sc.eq_ignore_ascii_case("S"))
            {
                let filtered_concepts =
                    filter_and_enrich_concepts(vec![concept.clone()], parameters, record_counts);
                if !filtered_concepts.is_empty() {
                    let filtered_concept = &filtered_concepts[0];
                    let score = sr.score.unwrap_or(0.0);

                    // Keep the highest score for each concept_id
                    if let Some((_, existing_score)) =
                        all_concepts.get(&filtered_concept.concept_id)
                    {
                        if score > *existing_score {
                            all_concepts.insert(
                                filtered_concept.concept_id,
                                (filtered_concept.clone(), score),
                            );
                        }
                    } else {
                        all_concepts.insert(
                            filtered_concept.concept_id,
                            (filtered_concept.clone(), score),
                        );
                    }
                }
            } else {
                let standard_concepts = standard_mappings
                    .get(&concept.concept_id)
                    .cloned()
                    .unwrap_or_default();
                let filtered_standard_concepts =
                    filter_and_enrich_concepts(standard_concepts, parameters, record_counts);
                for std_concept in filtered_standard_concepts {
                    let score = sr.score.unwrap_or(0.0);

                    // Keep the highest score for each concept_id
                    if let Some((_, existing_score)) = all_concepts.get(&std_concept.concept_id) {
                        if score > *existing_score {
                            all_concepts.insert(std_concept.concept_id, (std_concept, score));
                        }
                    } else {
                        all_concepts.insert(std_concept.concept_id, (std_concept, score));
                    }
                }
            }
        }
    }

    // Second pass: merge with existing concept_map, keeping best scores
    for (concept_id, (concept, score)) in all_concepts {
        if let Some((_, existing_score)) = concept_map.get(&concept_id) {
            if score > *existing_score {
                concept_map.insert(concept_id, (concept, score));
            }
        } else {
            concept_map.insert(concept_id, (concept, score));
        }
    }

    // Third pass: if over limit, keep only top N by score
    if concept_map.len() > limit {
        let mut concepts_vec: Vec<(i32, (Concept, f64))> = concept_map.drain().collect();
        concepts_vec.sort_by(|a, b| {
            b.1.1
                .partial_cmp(&a.1.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        concepts_vec.truncate(limit);
        *concept_map = concepts_vec.into_iter().collect();
    }
}

#[get("/api/search_standard")]
async fn search_standard(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
) -> Result<Json<Vec<SearchResponse>>, Error> {
    let cache_key = search_cache_key(&parameters, 25);
    let parameters = parameters.into_inner();
    let state_for_cache = state.clone();
    let result = state
        .search_standard_cache
        .try_get_with(cache_key, async move {
            search_standard_uncached(Query(parameters), state_for_cache)
                .await
                .map_err(|err| err.to_string())
        })
        .await
        .map_err(|err| ErrorInternalServerError(err.to_string()))?;

    Ok(Json(result))
}

async fn search_standard_uncached(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
) -> Result<Vec<SearchResponse>, Error> {
    let limit = parameters.limit.unwrap_or(25) as usize;
    let mut query_string = format!("q={}", parameters.q);
    if let Some(exclude_vocab_ids) = &parameters.exclude_vocabulary_id {
        let exclude_vocab_str = exclude_vocab_ids.join(",");
        query_string.push_str(&format!(
            "&exclude_vocabulary_id={}&limit=250",
            exclude_vocab_str
        ));
    }
    let (main_search_results, synonyms_search_results) = tokio::try_join!(
        search(
            Query::from_query(&query_string)?,
            state.clone(),
            CONCEPT_COLLECTION,
        ),
        search(
            Query::from_query(&query_string)?,
            state.clone(),
            SYNONYMS_COLLECTION,
        ),
    )?;

    // Resolve each non-standard source once across both collections.
    let source_ids = non_standard_source_ids(&main_search_results, &synonyms_search_results);
    let standard_mappings = if source_ids.is_empty() {
        HashMap::new()
    } else {
        let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
        db::map_to_standard_batch(&pg_client, &source_ids).await?
    };

    let mut concept_map: HashMap<i32, (Concept, f64)> = HashMap::new();
    for results in [main_search_results, synonyms_search_results] {
        process_search_results(
            results,
            &mut concept_map,
            &parameters,
            &state.concept_record_counts,
            &standard_mappings,
            limit,
        );
    }

    let mut grouped_concepts: HashMap<String, (f64, Vec<Concept>)> = HashMap::new();

    for (concept, score) in concept_map.into_values() {
        let name_lower = concept.concept_name.to_lowercase();

        if let Some((existing_score, concepts)) = grouped_concepts.get_mut(&name_lower) {
            if score > *existing_score {
                *existing_score = score;
            }
            concepts.push(concept);
        } else {
            grouped_concepts.insert(name_lower, (score, vec![concept]));
        }
    }

    let mut search_responses: Vec<SearchResponse> = grouped_concepts
        .into_iter()
        .map(|(name_lower, (score, concepts))| SearchResponse {
            concept_name: concepts[0].concept_name.clone(),
            concept_name_lower: name_lower,
            score: Some(score),
            concepts,
        })
        .collect();

    search_responses.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(search_responses)
}

#[get("/api/search")]
async fn search_api(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
) -> Result<Json<Vec<SearchResponse>>, Error> {
    let cache_key = search_cache_key(&parameters, 100);
    let state_for_cache = state.clone();
    let result = state
        .search_cache
        .try_get_with(cache_key, async move {
            search(parameters, state_for_cache, CONCEPT_COLLECTION)
                .await
                .map_err(|err| err.to_string())
        })
        .await
        .map_err(|err| ErrorInternalServerError(err.to_string()))?;
    Ok(Json(result))
}

async fn search(
    parameters: Query<Parameters>,
    state: Data<StateWrapper>,
    collection_name: &str,
) -> Result<Vec<SearchResponse>, Error> {
    let client = &state.qdrant_client;
    let input = parameters.q.trim();
    let lowercase_input = input.to_lowercase();
    info!("Received search request for {:?}", input);
    let opt_existing = if collection_name == CONCEPT_COLLECTION {
        state.concept_index.get(lowercase_input.as_str())
    } else {
        None
    };
    let mut ids: Vec<String> = Vec::new();
    if let Some(existing) = opt_existing {
        existing.iter().for_each(|x| ids.push(x.to_string()));
    } else {
        // Return the connection before waiting on Qdrant or embedding generation.
        let concepts = {
            let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
            match input.parse::<i32>() {
                Ok(id) => db::get_concept_name_by_number(&pg_client, id).await?,
                Err(_) => db::get_concept_name_by_string(&pg_client, input).await?,
            }
        };

        if !concepts.is_empty() {
            for c in concepts {
                let lower = c.to_lowercase();
                info!("{:?} resolved to {:?} in vocabulary", input, lower);
                let res = if collection_name == CONCEPT_COLLECTION {
                    state.concept_index.get(lower.as_str())
                } else {
                    None
                };
                if let Some(item) = res {
                    item.iter().for_each(|x| ids.push(x.to_string()))
                } else {
                    let results: Vec<RetrievedPoint> =
                        find_by_concept_name_lower(client, lower, collection_name)
                            .await
                            .map_err(|e| {
                                actix_web::error::ErrorInternalServerError(e.to_string())
                            })?;
                    for point in &results {
                        if let Some(PointIdOptions::Uuid(id)) = point
                            .id
                            .as_ref()
                            .and_then(|pid| pid.point_id_options.as_ref())
                        {
                            ids.push(id.to_string());
                        }
                    }
                }
            }
        } else {
            let limit = parameters.limit.unwrap_or(100);
            // Request more results from qdrant to account for filtering
            let search_limit = 250;
            let recommendations = recommend(
                input,
                client,
                &state.openai_client,
                &state.embedding_cache,
                search_limit,
                collection_name,
            )
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
            let mut groups = SearchResultGroups::default();
            for sp in recommendations {
                let mut concept: SearchResponse = SearchResponse::from(sp);
                // Apply filters after retrieval due to performance issues with filtering in qdrant
                concept.concepts = filter_and_enrich_concepts(
                    concept.concepts,
                    &parameters,
                    &state.concept_record_counts,
                );
                if concept.concepts.is_empty() {
                    continue;
                }
                groups.push(concept);
            }
            let mut to_return = groups.into_results();
            // Sort by score descending and apply limit
            to_return.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if to_return.len() > limit as usize {
                to_return.truncate(limit as usize);
            }
            return Ok(to_return);
        }
    }
    let points: Vec<PointId> = ids.iter().map(|id| PointId::from(id.as_str())).collect();
    create_response_from_vector_db_ids(
        client,
        points,
        &parameters,
        collection_name,
        &state.concept_record_counts,
    )
    .await
}

#[get("/api/concepts/{id}")]
async fn get_concept_by_id(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {}", id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut concept = db::get_concept_by_id(&pg_client, id).await?;
    concept.record_count = state.concept_record_counts.get(&id).copied().unwrap_or(0);
    Ok(HttpResponse::Ok().json([concept]))
}

#[get("/api/concepts/{id}/relationships")]
async fn get_concept_relationships(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {} relationships", id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut concepts = db::get_concept_relationships(&pg_client, id).await?;

    // Enrich with record counts
    for concept in &mut concepts {
        concept.record_count = state
            .concept_record_counts
            .get(&concept.concept_id)
            .copied()
            .unwrap_or(0);
    }

    Ok(HttpResponse::Ok().json(concepts))
}

#[get("/api/concepts/{id}/phoebe")]
async fn get_concept_phoebe(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {} phoebe", id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut concepts = db::get_concept_phoebe(&pg_client, id).await?;

    // Enrich with record counts
    for concept in &mut concepts {
        concept.record_count = state
            .concept_record_counts
            .get(&concept.concept_id)
            .copied()
            .unwrap_or(0);
    }

    Ok(HttpResponse::Ok().json(concepts))
}

#[derive(Debug, Deserialize)]
struct BulkPhoebeRequest {
    ids: Vec<i32>,
}

#[post("/api/concepts/phoebe/bulk")]
async fn get_concept_phoebe_bulk(
    request: Json<BulkPhoebeRequest>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let mut ids = request.ids.clone();
    ids.sort_unstable();
    ids.dedup();

    if ids.is_empty() {
        return Ok(HttpResponse::Ok().json(Vec::<PhoebeResponse>::new()));
    }
    if ids.len() > 500 {
        return Ok(HttpResponse::BadRequest().json("Too many IDs: maximum 500 per request"));
    }

    info!("Get phoebe bulk for {} concepts", ids.len());
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let mut phoebe_map = db::get_bulk_phoebe(&pg_client, &ids).await?;

    for results in phoebe_map.values_mut() {
        for concept in results.iter_mut() {
            concept.record_count = state
                .concept_record_counts
                .get(&concept.concept_id)
                .copied()
                .unwrap_or(0);
        }
    }

    let response: Vec<PhoebeResponse> = ids
        .iter()
        .map(|&id| PhoebeResponse {
            concept_id: id,
            results: phoebe_map.remove(&id).unwrap_or_default(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/concepts/{id}/definition")]
async fn get_concept_definition(
    path: web::Path<i32>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!("Get concept {} definition", id);
    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let concept = db::get_concept_by_id(&pg_client, id).await?;
    let def = match get_umls_definition_from_nlm(&concept.concept_name).await {
        Ok(Some(definition)) => definition,
        Ok(None) => "No definition available".to_string(),
        Err(e) => {
            log::warn!("UMLS lookup failed: {}", e);
            "No definition available".to_string()
        }
    };
    Ok(HttpResponse::Ok().json(def))
}

#[get("/api/concepts/{id}/expand")]
async fn get_concept_expand(
    path: web::Path<i32>,
    params: Query<ExpandParams>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    info!(
        "Get concept {} expand with params: childlevels={:?}, parentlevels={:?}",
        id, params.childlevels, params.parentlevels
    );

    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;
    let expander = ConceptExpander::new(
        &pg_client,
        &state.expand_cache,
        &state.children_cache,
        &state.parents_cache,
        &state.concept_record_counts,
    );
    let expand_response = expander
        .expand(id, params.childlevels, params.parentlevels)
        .await?;

    Ok(HttpResponse::Ok().json(expand_response))
}

async fn create_response_from_vector_db_ids(
    client: &Qdrant,
    points: Vec<PointId>,
    parameters: &Parameters,
    collection_name: &str,
    record_counts: &HashMap<i32, i64>,
) -> Result<Vec<SearchResponse>, Error> {
    let search_result = retrieve_point_from_db(client, points, collection_name)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let limit = parameters.limit.unwrap_or(100);

    // Extract vector from the first matched point to use for search
    let search_vector = if let Some(first_point) = search_result.first() {
        if let Some(vectors) = &first_point.vectors {
            match vectors.vectors_options.as_ref() {
                Some(qdrant::vectors_output::VectorsOptions::Vector(vector)) => {
                    match vector.clone().into_vector() {
                        qdrant::vector_output::Vector::Dense(dense) => Some(dense.data),
                        _ => None,
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    // Use Search API instead of Query/Recommend API for consistent results
    let neighbours = if let Some(vector) = search_vector {
        client
            .search_points(
                SearchPointsBuilder::new(collection_name, vector, 500)
                    .with_payload(true)
                    .score_threshold(0.50),
            )
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
            .result
    } else {
        Vec::new()
    };

    // Collect the point IDs from search_result to exclude them from neighbours
    // (they're already being returned and would cause duplicates)
    let search_result_ids: std::collections::HashSet<String> = search_result
        .iter()
        .filter_map(|p| {
            p.id.as_ref().and_then(|id| {
                if let Some(PointIdOptions::Uuid(uuid)) = &id.point_id_options {
                    Some(uuid.clone())
                } else {
                    None
                }
            })
        })
        .collect();

    let mut groups = SearchResultGroups::default();

    // Add search_result items first (these are the exact matches)
    for retrieved_point in search_result {
        let mut concept = SearchResponse::from(retrieved_point);
        concept.concepts = filter_and_enrich_concepts(concept.concepts, parameters, record_counts);
        if concept.concepts.is_empty() {
            continue;
        }
        groups.push(concept);
    }

    // Add neighbours, but exclude items that were already in search_result
    for scored_point in neighbours {
        // Skip if this point was already added from search_result
        if let Some(id) = &scored_point.id
            && let Some(PointIdOptions::Uuid(uuid)) = &id.point_id_options
            && search_result_ids.contains(uuid)
        {
            continue;
        }
        let mut concept = SearchResponse::from(scored_point);
        // Apply filters after retrieval due to performance issues with filtering in qdrant
        concept.concepts = filter_and_enrich_concepts(concept.concepts, parameters, record_counts);
        if concept.concepts.is_empty() {
            continue;
        }
        groups.push(concept);
    }

    let mut to_return = groups.into_results();
    // Sort by score descending and apply limit
    to_return.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if to_return.len() > limit as usize {
        to_return.truncate(limit as usize);
    }

    Ok(to_return)
}

async fn find_by_concept_name_lower(
    client: &Qdrant,
    concept_name_lower: String,
    collection: &str,
) -> Result<Vec<RetrievedPoint>, anyhow::Error> {
    Ok(client
        .scroll(
            ScrollPointsBuilder::new(collection).filter(Filter::must([Condition {
                condition_one_of: Some(ConditionOneOf::Field(qdrant::FieldCondition {
                    key: "concept_name_lower".to_string(),
                    r#match: Some(qdrant::Match {
                        match_value: Some(concept_name_lower.to_string().into()),
                    }),
                    range: None,
                    geo_bounding_box: None,
                    geo_radius: None,
                    values_count: None,
                    geo_polygon: None,
                    datetime_range: None,
                    is_empty: None,
                    is_null: None,
                })),
            }])),
        )
        .await?
        .result)
}

async fn retrieve_point_from_db(
    client: &Qdrant,
    points: Vec<PointId>,
    collection: &str,
) -> Result<Vec<RetrievedPoint>, anyhow::Error> {
    Ok(client
        .get_points(
            GetPointsBuilder::new(collection, points)
                .with_vectors(true)
                .with_payload(true),
        )
        .await?
        .result)
}

async fn recommend(
    input: &str,
    client: &Qdrant,
    openai_client: &async_openai::Client<async_openai::config::OpenAIConfig>,
    embedding_cache: &Cache<String, Vec<f32>>,
    limit: u64,
    collection_name: &str,
) -> Result<Vec<ScoredPoint>, anyhow::Error> {
    let vector = fetch_embeddings_cached(openai_client, input, embedding_cache).await?;
    Ok(client
        .search_points(SearchPointsBuilder::new(collection_name, vector, limit).with_payload(true))
        .await?
        .result)
}

fn filter_and_enrich_concepts(
    concepts: Vec<Concept>,
    parameters: &Parameters,
    record_counts: &HashMap<i32, i64>,
) -> Vec<Concept> {
    concepts
        .into_iter()
        .filter(|concept| {
            // Filter by vocabulary_id
            if let Some(vocab_ids) = &parameters.vocabulary_id
                && !vocab_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&concept.vocabulary_id))
            {
                return false;
            }

            // Exclude by vocabulary_id (substring match), wil cause ICD to filter ICD-N and RxNorm to filter also RxNorm Extension
            if let Some(exclude_vocab_ids) = &parameters.exclude_vocabulary_id
                && exclude_vocab_ids.iter().any(|id| {
                    concept
                        .vocabulary_id
                        .to_lowercase()
                        .contains(&id.to_lowercase())
                })
            {
                return false;
            }

            // Filter by standard_concept
            if let Some(std_concept) = &parameters.standard_concept {
                match concept.standard_concept.as_ref() {
                    Some(sc) if sc == std_concept => {}
                    None if std_concept.is_empty() => {}
                    _ => return false,
                }
            }

            // Filter by domain_id
            if let Some(domain_ids) = &parameters.domain_id
                && !domain_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&concept.domain_id))
            {
                return false;
            }

            // Filter by concept_class_id
            if let Some(class_ids) = &parameters.concept_class_id
                && !class_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&concept.concept_class_id))
            {
                return false;
            }

            true
        })
        .map(|mut concept| {
            // Enrich with record count
            concept.record_count = record_counts.get(&concept.concept_id).copied().unwrap_or(0);
            concept
        })
        .collect()
}

#[post("/api/conceptsets/analyze")]
async fn analyze_concept_set(
    request: Json<ConceptSetValidationRequest>,
    state: Data<StateWrapper>,
) -> Result<HttpResponse, Error> {
    info!("Received concept set analysis request");
    let concept_set = &request.concept_set;

    let pg_client = state.pg_pool.get().await.map_err(PgError::PoolError)?;

    let analysis_result = validation::analyze_concept_set(
        concept_set,
        &pg_client,
        Some(&state.qdrant_client),
        Some(&state.concept_index),
        Some(&state.concept_record_counts),
    )
    .await
    .unwrap_or_else(|e| {
        let mut error_result = validation::ValidationResult::new();
        error_result.add_error(format!("Database error during analysis: {}", e));
        error_result
    });

    Ok(HttpResponse::Ok().json(analysis_result.to_json()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn concept(id: i32, standard: Option<&str>, domain: &str) -> Concept {
        Concept {
            concept_id: id,
            concept_name: format!("Concept {id}"),
            domain_id: domain.into(),
            vocabulary_id: "SNOMED".into(),
            concept_class_id: "Clinical Finding".into(),
            standard_concept: standard.map(str::to_string),
            concept_code: id.to_string(),
            invalid_reason: None,
            valid_start_date: None,
            valid_end_date: None,
            record_count: 0,
        }
    }

    fn result(score: f64, concepts: Vec<Concept>) -> SearchResponse {
        SearchResponse {
            concept_name: "result".into(),
            concept_name_lower: "result".into(),
            score: Some(score),
            concepts,
        }
    }

    fn named_result(name: &str, score: f64, concepts: Vec<Concept>) -> SearchResponse {
        SearchResponse {
            concept_name: name.into(),
            concept_name_lower: name.to_lowercase(),
            score: Some(score),
            concepts,
        }
    }

    #[test]
    fn indexed_grouping_preserves_order_first_score_and_all_concepts() {
        let mut groups = SearchResultGroups::default();
        groups.push(named_result(
            "Exact",
            1.0,
            vec![concept(1, None, "Condition")],
        ));
        groups.push(named_result(
            "Other",
            0.8,
            vec![concept(2, None, "Condition")],
        ));
        groups.push(named_result(
            "EXACT",
            0.9,
            vec![concept(3, None, "Condition")],
        ));
        let results = groups.into_results();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].concept_name, "Exact");
        assert_eq!(results[0].score, Some(1.0));
        assert_eq!(
            results[0]
                .concepts
                .iter()
                .map(|c| c.concept_id)
                .collect::<Vec<_>>(),
            vec![1, 3]
        );
        assert_eq!(results[1].concept_name, "Other");
    }

    #[test]
    fn indexed_grouping_matches_original_scan_when_names_change_and_collide() {
        // Names can differ from the surviving standard concept after filtering.
        let mut standard = concept(10, Some("S"), "Condition");
        standard.concept_name = "Canonical".into();
        let candidates = [
            named_result("Alias", 0.9, vec![concept(1, None, "Condition")]),
            named_result("Alias", 0.8, vec![standard.clone()]),
            named_result("Canonical", 1.0, vec![standard]),
            named_result("ALIAS", 0.7, vec![concept(2, None, "Condition")]),
            named_result("Other", 0.6, vec![concept(3, None, "Condition")]),
        ];
        // Exercise both collision orders, revisiting old/new names, and duplicates.
        for mut sequence in 0..candidates.len().pow(5) {
            let mut expected: Vec<SearchResponse> = Vec::new();
            let mut actual = SearchResultGroups::default();
            for _ in 0..5 {
                let mut incoming = candidates[sequence % candidates.len()].clone();
                sequence /= candidates.len();
                actual.push(incoming.clone());
                let mut found = false;
                for existing in &mut expected {
                    if existing.concept_name_lower == incoming.concept_name_lower {
                        existing.append_concepts(&mut incoming.concepts);
                        found = true;
                    }
                }
                if !found {
                    expected.push(incoming);
                }
            }
            assert_eq!(
                serde_json::to_value(actual.into_results()).unwrap(),
                serde_json::to_value(expected).unwrap(),
            );
        }
    }

    #[test]
    fn mapping_sources_are_unique_across_collections_and_exclude_standard_concepts() {
        let main = vec![result(
            0.8,
            vec![
                concept(1, None, "Condition"),
                concept(2, Some("S"), "Condition"),
                concept(3, Some("s"), "Condition"),
            ],
        )];
        let synonyms = vec![result(
            0.9,
            vec![
                concept(1, None, "Condition"),
                concept(4, Some("C"), "Condition"),
            ],
        )];
        let mut ids = non_standard_source_ids(&main, &synonyms);
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 4]);
        assert!(non_standard_source_ids(&[], &[]).is_empty());
    }

    #[test]
    fn batched_mappings_preserve_best_scores_filters_counts_and_limit() {
        let parameters = Query::<Parameters>::from_query("q=test&domain_id=Condition").unwrap();
        let mappings = HashMap::from([
            (
                1,
                vec![
                    concept(10, Some("S"), "Condition"),
                    concept(11, Some("S"), "Drug"),
                ],
            ),
            (
                2,
                vec![
                    concept(10, Some("S"), "Condition"),
                    concept(12, Some("S"), "Condition"),
                ],
            ),
        ]);
        let counts = HashMap::from([(10, 123), (12, 456)]);
        let mut concepts = HashMap::new();
        process_search_results(
            vec![
                result(0.7, vec![concept(1, None, "Condition")]),
                result(0.8, vec![concept(10, Some("S"), "Condition")]),
                result(0.6, vec![concept(13, Some("S"), "Condition")]),
                result(1.0, vec![concept(99, None, "Condition")]),
            ],
            &mut concepts,
            &parameters,
            &counts,
            &mappings,
            2,
        );
        assert_eq!(concepts.len(), 2);
        assert_eq!(concepts[&10].1, 0.8);
        assert!(concepts.contains_key(&13));
        process_search_results(
            vec![result(
                0.9,
                vec![concept(2, None, "Condition"), concept(1, None, "Condition")],
            )],
            &mut concepts,
            &parameters,
            &counts,
            &mappings,
            2,
        );
        assert_eq!(concepts.len(), 2);
        assert_eq!(concepts[&10].1, 0.9);
        assert_eq!(concepts[&12].1, 0.9);
        assert_eq!(concepts[&10].0.record_count, 123);
        assert_eq!(concepts[&12].0.record_count, 456);
    }
}
