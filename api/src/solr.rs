use anyhow::{Context, Result};
use log::info;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SolrResponse {
    response: SolrResponseBody,
}

#[derive(Debug, Deserialize)]
struct SolrResponseBody {
    #[serde(rename = "numFound")]
    num_found: u64,
    docs: Vec<SolrDoc>,
}

#[derive(Debug, Deserialize)]
pub struct SolrDoc {
    pub concept_id: i32,
    pub concept_name: String,
    pub domain_id: String,
    pub vocabulary_id: String,
    pub concept_class_id: String,
    pub standard_concept: Option<String>,
    pub concept_code: String,
    pub invalid_reason: Option<String>,
    #[serde(default)]
    pub record_count: i64,
}

pub struct SolrFilters {
    pub vocabulary_id: Option<Vec<String>>,
    pub exclude_vocabulary_id: Option<Vec<String>>,
    pub domain_id: Option<Vec<String>>,
    pub concept_class_id: Option<Vec<String>>,
    pub standard_concept: Option<String>,
}

pub struct SolrClient {
    client: Client,
    base_url: String,
}

impl SolrClient {
    pub fn new(solr_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to build Solr HTTP client")?;

        Ok(Self {
            client,
            base_url: solr_url.trim_end_matches('/').to_string(),
        })
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/admin/ping", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .context("Solr health check failed")?
            .error_for_status()
            .context("Solr health check returned error status")?;
        Ok(())
    }

    pub async fn search(&self, query: &str, filters: &SolrFilters, limit: u64) -> Result<Vec<SolrDoc>> {
        info!("Solr lexical search for: {:?}", query);

        let mut params: Vec<(&str, String)> = vec![
            ("q", query.to_string()),
            ("rows", limit.to_string()),
            ("wt", "json".to_string()),
        ];

        // Build filter queries
        if let Some(vocab_ids) = &filters.vocabulary_id {
            let fq = vocab_ids
                .iter()
                .map(|v| format!("\"{}\"", v))
                .collect::<Vec<_>>()
                .join(" OR ");
            params.push(("fq", format!("vocabulary_id:({})", fq)));
        }

        if let Some(exclude_vocab_ids) = &filters.exclude_vocabulary_id {
            for ev in exclude_vocab_ids {
                params.push(("fq", format!("-vocabulary_id:\"{}\"", ev)));
            }
        }

        if let Some(domain_ids) = &filters.domain_id {
            let fq = domain_ids
                .iter()
                .map(|d| format!("\"{}\"", d))
                .collect::<Vec<_>>()
                .join(" OR ");
            params.push(("fq", format!("domain_id:({})", fq)));
        }

        if let Some(class_ids) = &filters.concept_class_id {
            let fq = class_ids
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(" OR ");
            params.push(("fq", format!("concept_class_id:({})", fq)));
        }

        if let Some(std) = &filters.standard_concept {
            params.push(("fq", format!("standard_concept:\"{}\"", std)));
        }

        // Build URL with query parameters using form_urlencoded for proper encoding
        let query_string: String = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params.iter().map(|(k, v)| (*k, v.as_str())))
            .finish();
        let url = format!("{}/select?{}", self.base_url, query_string);

        let response: SolrResponse = self
            .client
            .get(&url)
            .send()
            .await
            .context("Solr query request failed")?
            .error_for_status()
            .context("Solr returned error status")?
            .json()
            .await
            .context("Failed to deserialize Solr response")?;

        info!(
            "Solr returned {} results (of {} total)",
            response.response.docs.len(),
            response.response.num_found
        );
        Ok(response.response.docs)
    }
}
