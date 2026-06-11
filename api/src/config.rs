use confik::Configuration;
use serde::Deserialize;

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(Configuration, Clone, Deserialize)]
pub struct Configs {
    pub server_addr: String,
    pub qdrant_uri: String,
    pub vectordb_data_path: String,
    #[serde(deserialize_with = "deserialize_comma_separated")]
    pub cors_origins: Vec<String>,
    #[confik(from = DbConfig)]
    pub pg: deadpool_postgres::Config,
    pub expand_cache_max_entries: u64,
    pub cache_ttl_days: u64,
    pub search_cache_max_entries: u64,
    pub embedding_cache_max_entries: u64,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:8080".to_string(),
            qdrant_uri: "http://localhost:6334".to_string(),
            vectordb_data_path: "sample_data.txt".to_string(),
            cors_origins: vec!["http://localhost:5173".to_string()],
            pg: deadpool_postgres::Config::default(),
            expand_cache_max_entries: 64,
            cache_ttl_days: 35,
            search_cache_max_entries: 1000,
            embedding_cache_max_entries: 128_000,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct DbConfig(deadpool_postgres::Config);

impl From<DbConfig> for deadpool_postgres::Config {
    fn from(value: DbConfig) -> Self {
        value.0
    }
}

impl confik::Configuration for DbConfig {
    type Builder = Option<Self>;
}
