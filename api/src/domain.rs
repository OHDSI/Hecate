use chrono::NaiveDate;
use qdrant_client::qdrant::{RetrievedPoint, ScoredPoint};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub concept_name: String,
    pub concept_name_lower: String,
    pub score: Option<f64>,
    pub concepts: Vec<Concept>,
}

impl SearchResponse {
    pub(crate) fn append_concepts(&mut self, additional_concepts: &mut Vec<Concept>) {
        self.concepts.append(additional_concepts)
    }
}

impl From<ScoredPoint> for SearchResponse {
    fn from(item: ScoredPoint) -> Self {
        let payload = serde_json::to_string(&item.payload).unwrap();
        let res: Result<SearchResponse, _> = serde_json::from_str(&payload);
        if res.is_ok() {
            let mut concept: SearchResponse = res.unwrap();
            concept.score = Some(item.score as f64);
            concept
        } else {
            dbg!("{?}", item.payload);
            SearchResponse {
                concept_name: "String".parse().unwrap(),
                concept_name_lower: "String".parse().unwrap(),
                score: Some(0f64),
                concepts: Vec::new(),
            }
        }
    }
}

impl From<RetrievedPoint> for SearchResponse {
    fn from(item: RetrievedPoint) -> Self {
        let payload = serde_json::to_string(&item.payload).unwrap();
        let mut concept: SearchResponse = serde_json::from_str(&payload).unwrap();
        concept.score = Some(1f64);
        concept
    }
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
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
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "related_concept_dto)")]
pub struct RelatedConcept {
    pub relationship_id: String,
    pub concept_id: i32,
    pub concept_name: String,
    pub vocabulary_id: String,
}
