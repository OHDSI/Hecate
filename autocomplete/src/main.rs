use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use actix_cors::Cors;
use actix_web::web::{Data, Query};
use actix_web::{App, HttpResponse, HttpServer, get};
use anyhow::Context;
use indicium::simple::{AutocompleteType, SearchIndex, SearchIndexBuilder};
use log::{LevelFilter, info};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Parameters {
    q: String,
}

#[get("/api/autocomplete")]
async fn lookup(
    parameters: Query<Parameters>,
    search_index: Data<SearchIndex<u32>>,
) -> HttpResponse {
    let q = parameters.q.as_str();
    let suggestions = search_index.autocomplete(q);
    HttpResponse::Ok().json(suggestions)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    info!("Starting Autocomplete Service");
    let app_data = load_app_data()?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://hecate.pantheon-hds.com")
            .allowed_origin("http://localhost:5173");
        App::new()
            .wrap(cors)
            .service(lookup)
            .app_data(app_data.clone())
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
    .context("HTTP server failed")
}

fn load_app_data() -> anyhow::Result<Data<SearchIndex<u32>>> {
    let items = read_lines_from_file("./sample_values.txt")?;
    let autocomplete = create_search_index(items);
    Ok(Data::new(autocomplete))
}

fn read_lines_from_file(filename: &str) -> anyhow::Result<HashSet<String>> {
    let path = Path::new(filename);
    info!("Reading lines from file: {}", path.display());
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let buf = BufReader::new(file);
    buf.lines()
        .collect::<Result<HashSet<String>, _>>()
        .context("failed to read line from file")
}

fn create_search_index(values: HashSet<String>) -> SearchIndex<u32> {
    info!("Creating search index");
    let mut search_index = init_search_index();
    for (i, value) in values.into_iter().enumerate() {
        search_index.insert(&(i as u32), &value);
    }
    search_index
}

fn init_search_index() -> SearchIndex<u32> {
    SearchIndexBuilder::default()
        .split_pattern(None)
        .exclude_keywords(None)
        .fuzzy_minimum_score(0.5)
        .fuzzy_length(3)
        .min_keyword_len(3)
        .max_keyword_len(128)
        .max_string_len(Some(256))
        .autocomplete_type(AutocompleteType::Keyword)
        .max_search_results(10)
        .build()
}
