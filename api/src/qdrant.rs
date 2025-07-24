use crate::api::COLLECTION_NAME;
use log::info;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{PayloadIncludeSelector, PointId, RetrievedPoint, ScrollPointsBuilder};
use std::fs;
use uuid::Uuid;

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

pub async fn get_all_id_value_pairs(client: &Qdrant) -> Vec<(Uuid, String)> {
    let points = get_all_points(client, COLLECTION_NAME).await;
    let mut pairs: Vec<(Uuid, String)> = Vec::new();
    for point in points {
        let index: PointIdOptions = point.clone().id.unwrap().point_id_options.unwrap();
        let concept_name = point
            .clone()
            .payload
            .get("concept_name_lower")
            .unwrap()
            .to_string();
        if let PointIdOptions::Uuid(id) = index {
            let v = rem_first_and_last(concept_name.as_str()).to_string();
            let uuid = Uuid::try_parse(id.as_str());
            let k = uuid.unwrap();
            pairs.push((k, v));
        }
    }
    pairs
}

async fn get_all_points(client: &Qdrant, collection_name: &str) -> Vec<RetrievedPoint> {
    let mut result: Vec<RetrievedPoint> = Vec::new();
    let mut has_next = true;
    let mut offset: Option<PointId> = None;
    while has_next {
        // while result.len() < 150000 {
        let mut spb = ScrollPointsBuilder::new(collection_name)
            .limit(5000)
            .with_payload(SelectorOptions::Include(PayloadIncludeSelector::from(
                vec![String::from("concept_name_lower")],
            )));
        if offset.is_some() {
            spb = spb.offset(offset.clone().unwrap())
        }

        let resp = client.scroll(spb).await.unwrap();
        resp.result.iter().for_each(|rp| result.push(rp.clone()));
        if resp.next_page_offset.is_some() {
            offset = resp.next_page_offset;
        } else {
            has_next = false;
        }
    }

    info!(
        "Collected {} concepts for {}",
        result.len(),
        &collection_name
    );
    result
}

#[allow(dead_code)]
async fn write_pairs_to_file(qdrant_client: &Qdrant) {
    let id_value_pairs = get_all_id_value_pairs(qdrant_client).await;
    let asdf = serde_json::to_string(&id_value_pairs).unwrap();
    fs::write("/Users/rowan/code/hecate/hecate-api/all_pairs.txt", asdf).unwrap();
}
