mod api;
mod config;
mod db;
mod domain;
mod embeddings;
mod errors;
mod qdrant;
mod umls;
mod utils;

use crate::api::{
    get_concept_by_id, get_concept_definition, get_concept_phoebe, get_concept_relationships,
    search,
};
use crate::config::Configs;
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use confik::{Configuration, EnvSource};
use deadpool_postgres::Pool;
use dotenvy::dotenv;
use log::{LevelFilter, info};
use qdrant_client::Qdrant;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use tokio_postgres::NoTls;
use uuid::Uuid;

struct StateWrapper {
    concept_index: HashMap<String, Vec<Uuid>>,
    pg_pool: Pool,
    qdrant_client: Qdrant,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    info!("Starting Hecate API!");
    info!("Init env");
    dotenv().ok();

    info!("Load configs");
    let config = Configs::builder()
        .override_with(EnvSource::new())
        .try_build()
        .unwrap();

    let state = create_state(&config).await.unwrap();

    HttpServer::new(move || {
        let mut cors = Cors::default();
        for origin in &config.cors_origins {
            cors = cors.allowed_origin(origin);
        }
        App::new()
            .wrap(cors)
            .service(search)
            .service(get_concept_by_id)
            .service(get_concept_relationships)
            .service(get_concept_definition)
            .service(get_concept_phoebe)
            .app_data(state.clone())
    })
    .bind(config.server_addr.clone())?
    .run()
    .await
}

async fn create_state(config: &Configs) -> Result<Data<StateWrapper>, Box<dyn Error>> {
    info!("Initializing Postgres pool");
    let pg_pool = config.pg.create_pool(None, NoTls)?;
    pg_pool
        .get()
        .await?
        .simple_query("SELECT 1")
        .await
        .expect("Postgres test query failed");

    info!("Initializing Qdrant client");
    let qdrant_client = Qdrant::from_url(&config.qdrant_uri).build()?;
    qdrant_client
        .health_check()
        .await
        .expect("Qdrant health check failed");

    let concept_index = load_concept_index(&config.vectordb_data_path)?;

    let state = Data::new(StateWrapper {
        concept_index,
        pg_pool,
        qdrant_client,
    });
    info!("App data loaded");
    Ok(state)
}

fn load_concept_index(
    vectordb_data_path: &str,
) -> Result<HashMap<String, Vec<Uuid>>, Box<dyn Error>> {
    info!(
        "Load all concept-vector_ids map from file: {}",
        vectordb_data_path
    );
    let bytes = fs::read_to_string(vectordb_data_path)?;
    let value_ids_map: HashMap<String, Vec<Uuid>> = serde_json::from_str(&bytes)?;
    info!("{} concept-vector_ids loaded", value_ids_map.len());
    Ok(value_ids_map)
}
