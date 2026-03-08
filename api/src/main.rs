mod api;
mod concept_graph;
mod config;
mod db;
mod domain;
mod embeddings;
mod errors;
mod umls;
mod utils;
mod solr;
mod validation;

use crate::api::{
    analyze_concept_set, get_concept_by_id, get_concept_definition, get_concept_expand,
    get_concept_phoebe, get_concept_relationships, search_api, search_lexical, search_standard,
};
use crate::config::Configs;
use crate::domain::{Concept, ExpandCacheKey, ExpandResponse};
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use anyhow::Context;
use confik::{Configuration, EnvSource};
use deadpool_postgres::Pool;
use dotenvy::dotenv;
use log::{LevelFilter, info};
use moka::future::Cache;
use qdrant_client::Qdrant;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio_postgres::NoTls;
use uuid::Uuid;

struct StateWrapper {
    concept_index: HashMap<String, Vec<Uuid>>,
    concept_record_counts: HashMap<i32, i64>,
    pg_pool: Pool,
    qdrant_client: Qdrant,
    solr_client: Option<crate::solr::SolrClient>,
    expand_cache: Cache<ExpandCacheKey, ExpandResponse>,
    children_cache: Cache<i32, Vec<Concept>>,
    parents_cache: Cache<i32, Vec<Concept>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::builder().filter_level(LevelFilter::Info).init();
    info!("Starting Hecate API!");

    info!("Load configs");
    let config = Configs::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Failed to load configuration");

    let state = create_state(&config)
        .await
        .expect("Failed to initialize application state");

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        for origin in &config.cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .service(search_standard)
            .service(search_api)
            .service(search_lexical)
            .service(get_concept_by_id)
            .service(get_concept_relationships)
            .service(get_concept_definition)
            .service(get_concept_phoebe)
            .service(get_concept_expand)
            .service(analyze_concept_set)
            .app_data(state.clone())
    })
    .bind(config.server_addr.clone())?
    .run()
    .await
}

async fn create_state(config: &Configs) -> anyhow::Result<Data<StateWrapper>> {
    info!("Initializing Postgres pool");
    let pg_pool = config
        .pg
        .create_pool(None, NoTls)
        .context("Failed to create Postgres pool")?;
    pg_pool
        .get()
        .await
        .context("Failed to get Postgres connection")?
        .simple_query("SELECT 1")
        .await
        .context("Postgres test query failed")?;

    info!("Initializing Qdrant client");
    let qdrant_client = Qdrant::from_url(&config.qdrant_uri)
        .build()
        .context("Failed to build Qdrant client")?;
    qdrant_client
        .health_check()
        .await
        .context("Qdrant health check failed")?;

    let solr_client = if let Some(solr_url) = &config.solr_url {
        info!("Initializing Solr client (url: {})", solr_url);
        let client = crate::solr::SolrClient::new(solr_url)
            .context("Failed to build Solr client")?;
        client
            .health_check()
            .await
            .context("Solr health check failed - is Solr running?")?;
        Some(client)
    } else {
        info!("Solr not configured (SOLR_URL not set), lexical search disabled");
        None
    };

    let concept_index = load_concept_index(&config.vectordb_data_path)?;
    let concept_record_counts = load_concept_record_counts()?;

    info!(
        "Initializing concept expansion cache (max_capacity: {}, ttl: {} days)",
        config.cache_max_capacity, config.cache_ttl_days
    );
    let expand_cache = Cache::builder()
        .max_capacity(config.cache_max_capacity)
        .time_to_live(Duration::from_secs(config.cache_ttl_days * 24 * 60 * 60))
        .build();

    info!("Initializing children and parents caches");
    let children_cache = Cache::builder()
        .max_capacity(5000) // More entries for individual lookups
        .time_to_live(Duration::from_secs(config.cache_ttl_days * 24 * 60 * 60))
        .build();

    let parents_cache = Cache::builder()
        .max_capacity(5000) // More entries for individual lookups
        .time_to_live(Duration::from_secs(config.cache_ttl_days * 24 * 60 * 60))
        .build();

    let state = Data::new(StateWrapper {
        concept_index,
        concept_record_counts,
        pg_pool,
        qdrant_client,
        solr_client,
        expand_cache,
        children_cache,
        parents_cache,
    });
    info!("App data loaded");
    Ok(state)
}

fn load_concept_index(vectordb_data_path: &str) -> anyhow::Result<HashMap<String, Vec<Uuid>>> {
    info!(
        "Load all concept-vector_ids map from file: {}",
        vectordb_data_path
    );
    let bytes = fs::read_to_string(vectordb_data_path)?;
    let value_ids_map: HashMap<String, Vec<Uuid>> = serde_json::from_str(&bytes)?;
    info!("{} concept-vector_ids loaded", value_ids_map.len());
    Ok(value_ids_map)
}

fn load_concept_record_counts() -> anyhow::Result<HashMap<i32, i64>> {
    let path = "ConceptRecordCounts.json";
    info!("Load concept record counts from file: {}", path);
    let bytes = fs::read_to_string(path)?;
    let record_counts: HashMap<i32, i64> = serde_json::from_str(&bytes)?;
    info!("{} concept record counts loaded", record_counts.len());
    Ok(record_counts)
}
