use chrono::NaiveDate;
use qdrant_client::qdrant::{RetrievedPoint, ScoredPoint};
use serde::{Deserialize, Serialize, Serializer};
use std::hash::Hash;
use tokio_pg_mapper_derive::PostgresMapper;

fn serialize_children<S>(children: &Vec<ExpandConcept>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if children.is_empty() {
        serializer.serialize_none()
    } else {
        children.serialize(serializer)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub concept_name: String,
    pub concept_name_lower: String,
    pub score: Option<f64>,
    pub concepts: Vec<Concept>,
}

impl SearchResponse {
    pub(crate) fn append_concepts(&mut self, additional_concepts: &mut Vec<Concept>) {
        self.concepts.append(additional_concepts);
        // Update concept_name to use standard concept name if available
        if let Some(standard_name) = self.find_standard_concept_name() {
            self.concept_name = standard_name;
            self.concept_name_lower = self.concept_name.to_lowercase();
        }
    }

    fn find_standard_concept_name(&self) -> Option<String> {
        self.concepts
            .iter()
            .find(|concept| concept.standard_concept.as_deref() == Some("S"))
            .map(|concept| concept.concept_name.clone())
    }
}

impl From<ScoredPoint> for SearchResponse {
    fn from(item: ScoredPoint) -> Self {
        let payload = serde_json::to_string(&item.payload).unwrap();
        let res: Result<SearchResponse, _> = serde_json::from_str(&payload);
        match res {
            Ok(mut concept) => {
                concept.score = Some(item.score as f64);
                if let Some(standard_name) = concept.find_standard_concept_name() {
                    concept.concept_name = standard_name;
                    concept.concept_name_lower = concept.concept_name.to_lowercase();
                }
                concept
            }
            Err(_) => {
                log::error!("Failed to deserialize ScoredPoint payload");
                SearchResponse {
                    concept_name: String::new(),
                    concept_name_lower: String::new(),
                    score: Some(0f64),
                    concepts: Vec::new(),
                }
            }
        }
    }
}

impl From<RetrievedPoint> for SearchResponse {
    fn from(item: RetrievedPoint) -> Self {
        let payload = serde_json::to_string(&item.payload).unwrap();
        let mut concept: SearchResponse = serde_json::from_str(&payload).unwrap();
        concept.score = Some(1f64);
        // Update concept_name to use standard concept name if available
        if let Some(standard_name) = concept.find_standard_concept_name() {
            concept.concept_name = standard_name;
            concept.concept_name_lower = concept.concept_name.to_lowercase();
        }
        concept
    }
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "concept")]
pub struct Concept {
    pub concept_id: i32,
    pub concept_name: String,
    pub domain_id: String,
    pub vocabulary_id: String,
    pub concept_class_id: String,
    pub standard_concept: Option<String>,
    pub concept_code: String,
    pub invalid_reason: Option<String>,
    pub valid_start_date: Option<NaiveDate>,
    pub valid_end_date: Option<NaiveDate>,
    #[serde(default)]
    pub record_count: i64,
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "related_concept_dto)")]
pub struct RelatedConcept {
    pub relationship_id: String,
    pub concept_id: i32,
    pub concept_name: String,
    pub vocabulary_id: String,
    #[serde(default)]
    pub record_count: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExpandConcept {
    pub concept_id: i32,
    pub concept_name: String,
    pub domain_id: String,
    pub vocabulary_id: String,
    pub concept_class_id: String,
    pub concept_code: String,
    pub invalid_reason: Option<String>,
    pub level: Option<i32>,
    #[serde(default)]
    pub record_count: i64,
    #[serde(serialize_with = "serialize_children")]
    pub children: Vec<ExpandConcept>,
}

impl From<Concept> for ExpandConcept {
    fn from(concept: Concept) -> Self {
        ExpandConcept {
            concept_id: concept.concept_id,
            concept_name: concept.concept_name,
            domain_id: concept.domain_id,
            vocabulary_id: concept.vocabulary_id,
            concept_class_id: concept.concept_class_id,
            concept_code: concept.concept_code,
            invalid_reason: concept.invalid_reason,
            level: None,
            record_count: concept.record_count,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExpandResponse {
    pub concepts: Vec<ExpandConcept>,
    pub childlevel: Option<i32>,
    pub parentlevel: Option<i32>,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpandCacheKey {
    pub concept_id: i32,
    pub child_levels: Option<i32>,
    pub parent_levels: Option<i32>,
}

impl ExpandCacheKey {
    pub fn new(concept_id: i32, child_levels: Option<i32>, parent_levels: Option<i32>) -> Self {
        Self {
            concept_id,
            child_levels,
            parent_levels,
        }
    }
}
