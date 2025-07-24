use confik::Configuration;
use serde::Deserialize;

#[derive(Default, Configuration, Clone)]
pub struct Configs {
    pub server_addr: String,
    pub qdrant_uri: String,
    pub vectordb_data_path: String,
    pub cors_origins: Vec<String>,
    #[confik(from = DbConfig)]
    pub pg: deadpool_postgres::Config,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct DbConfig(deadpool_postgres::Config);

impl From<DbConfig> for deadpool_postgres::Config {
    fn from(value: DbConfig) -> Self {
        value.0
    }
}

impl confik::Configuration for DbConfig {
    type Builder = Option<Self>;
}
