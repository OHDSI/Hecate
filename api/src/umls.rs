use log::{debug, error, warn};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct Unauthorized;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct NLMResponse {
    pub page_size: u16,
    pub page_number: u16,
    pub result: Option<NLMResponseResult>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct NLMResponseResult {
    pub class_type: String,
    pub rec_count: Option<u16>,
    pub entity_list: Vec<NLMConcept>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct NLMConcept {
    pub label: String,
    pub ui: String,
    pub sem_types: Vec<String>,
    pub sources: Vec<String>,
    pub sab: String,
    pub definition: Option<String>,
    pub full_match: bool,
}

pub async fn get_umls_definition_from_nlm(concept: String) -> Result<Option<String>, Unauthorized> {
    let api_key = env::var("UMLS_API_KEY").expect("UMLS_API_KEY must be set");
    let url = format!(
        "https://uts-ws.nlm.nih.gov/esearch/es/current?apiKey={}",
        api_key
    );

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body("{\"languages\":[],\"pageNumber\":1,\"pageSize\":1,\"returnType\":\"concept\",\"sabs\":[],\"searchString\":\"".to_owned() + &*concept +"\", \"exactSearch\":true,\"vocabulary\":[],\"semanticGroup\":[]}")
        .send()
        .await;

    let body: NLMResponse = match response {
        Ok(r) => {
            if r.status() == 200 {
                r.json().await.unwrap()
            } else if r.status() == 401 {
                return Err(Unauthorized);
            } else {
                warn!("Got a {} status for {}", r.status(), concept);
                return Ok(None);
            }
        }
        Err(_e) => {
            return Ok(None);
        }
    };

    debug!("{:?}", body);

    match body.result {
        Some(mut r) => {
            if r.entity_list.len() == 1 {
                let first = r.entity_list.swap_remove(0);
                Ok(first.definition)
            } else {
                error!("Something weird found for {}", concept);
                Ok(None)
            }
        }
        None => Ok(None),
    }
}
