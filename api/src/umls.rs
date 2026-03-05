use log::{debug, error, warn};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UmlsError {
    #[error("UMLS API returned 401 Unauthorized")]
    Unauthorized,
    #[error("UMLS API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("UMLS_API_KEY environment variable not set")]
    MissingApiKey,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NLMResponse {
    pub result: Option<NLMResponseResult>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NLMResponseResult {
    pub entity_list: Vec<NLMConcept>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NLMConcept {
    pub definition: Option<String>,
}

pub async fn get_umls_definition_from_nlm(concept: String) -> Result<Option<String>, UmlsError> {
    let api_key = std::env::var("UMLS_API_KEY").map_err(|_| UmlsError::MissingApiKey)?;
    let url = format!(
        "https://uts-ws.nlm.nih.gov/esearch/es/current?apiKey={}",
        api_key
    );

    let request_body = serde_json::json!({
        "languages": [],
        "pageNumber": 1,
        "pageSize": 1,
        "returnType": "concept",
        "sabs": [],
        "searchString": concept,
        "exactSearch": true,
        "vocabulary": [],
        "semanticGroup": []
    });

    let client = reqwest::Client::new();
    let response = match client
        .post(url)
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("UMLS request failed for {}: {}", concept, e);
            return Ok(None);
        }
    };

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(UmlsError::Unauthorized);
    }
    if !status.is_success() {
        warn!("Got a {} status for {}", status, concept);
        return Ok(None);
    }

    let body: NLMResponse = match response.json().await {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to parse UMLS response for {}: {}", concept, e);
            return Ok(None);
        }
    };

    debug!("{:?}", body);

    match body.result {
        Some(mut r) if r.entity_list.len() == 1 => Ok(r.entity_list.swap_remove(0).definition),
        Some(_) => {
            error!("Unexpected number of results for {}", concept);
            Ok(None)
        }
        None => Ok(None),
    }
}
