//! Exports concept name → Qdrant UUID mappings to a JSON file.
//!
//! Scrolls all points in the `meddra` Qdrant collection and writes a
//! `HashMap<String, Vec<Uuid>>` mapping lowercased concept names to their
//! Qdrant point UUIDs. The output file is used by the main API as the
//! concept search index (configured via `VECTORDB_DATA_PATH` in `.env`).
//!
//! # Usage
//!
//! ```sh
//! # From the api/ directory (reads QDRANT_URI from .env)
//! cargo run --bin write_pairs
//!
//! # Custom output path
//! cargo run --bin write_pairs -- my_output.txt
//! ```
//!
//! Output defaults to `all_pairs.txt` in the current directory.

use dotenvy::dotenv;
use log::info;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{PayloadIncludeSelector, PointId, ScrollPointsBuilder};
use std::collections::HashMap;
use std::fs;
use uuid::Uuid;

const CONCEPT_COLLECTION: &str = "meddra";

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let qdrant_uri = std::env::var("QDRANT_URI").expect("QDRANT_URI must be set");
    info!("Connecting to Qdrant at {}", &qdrant_uri);
    let client = Qdrant::from_url(&qdrant_uri).build().unwrap();

    info!("Scrolling all points from '{}'...", CONCEPT_COLLECTION);
    let mut pairs: HashMap<String, Vec<Uuid>> = HashMap::new();
    let mut offset: Option<PointId> = None;
    loop {
        let mut request = ScrollPointsBuilder::new(CONCEPT_COLLECTION)
            .limit(5000)
            .with_payload(SelectorOptions::Include(PayloadIncludeSelector::from(
                vec![String::from("concept_name_lower")],
            )));
        if let Some(ref o) = offset {
            request = request.offset(o.clone());
        }

        let resp = client.scroll(request).await.unwrap();
        for point in &resp.result {
            let Some(ref id) = point.id else { continue };
            let Some(PointIdOptions::Uuid(ref uuid_str)) = id.point_id_options else {
                continue;
            };
            let Some(value) = point.payload.get("concept_name_lower") else {
                continue;
            };
            let name = value.to_string();
            let name = name.trim_matches('"').to_string();
            let uuid = Uuid::try_parse(uuid_str).unwrap();
            pairs.entry(name).or_default().push(uuid);
        }
        info!("Collected {} unique concept names so far...", pairs.len());

        match resp.next_page_offset {
            Some(next) => offset = Some(next),
            None => break,
        }
    }

    let output = std::env::args().nth(1).unwrap_or("all_pairs.txt".into());
    info!("Writing {} entries to {}", pairs.len(), &output);
    let json = serde_json::to_string(&pairs).unwrap();
    fs::write(&output, json).unwrap();
    info!("Done.");
}
