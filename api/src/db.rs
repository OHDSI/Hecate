use crate::domain::{Concept, RelatedConcept};
use crate::errors::PgError;
use deadpool_postgres::Client;
use log::info;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_concept_name_by_number(
    client: &Client,
    input: i32,
) -> Result<Vec<String>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = include_str!("../sql/select_concept_for_numeric_input.sql");
    let stmt = client.prepare(stmt).await?;

    let results = client
        .query(&stmt, &[&input, &input.to_string()])
        .await?
        .iter()
        .map(|row| row.get("concept_name"))
        .collect::<Vec<String>>();

    Ok(results)
}

pub async fn get_concept_by_id(client: &Client, input: i32) -> Result<Concept, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = include_str!("../sql/select_concept_by_id.sql");
    let stmt = client.prepare(stmt).await?;

    let result = client
        .query(&stmt, &[&input])
        .await?
        .iter()
        .map(|row| Concept::from_row(row.clone()).unwrap())
        .next_back()
        .unwrap();

    Ok(result)
}

pub async fn get_concept_relationships(
    client: &Client,
    input: i32,
) -> Result<Vec<RelatedConcept>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = include_str!("../sql/select_related_concepts.sql");
    let stmt = client.prepare(stmt).await?;

    let results = client
        .query(&stmt, &[&input])
        .await?
        .iter()
        .map(|row| RelatedConcept::from_row(row.clone()).unwrap())
        .collect::<Vec<RelatedConcept>>();

    Ok(results)
}

pub async fn get_concept_phoebe(
    client: &Client,
    input: i32,
) -> Result<Vec<RelatedConcept>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = include_str!("../sql/select_phoebe_concepts.sql");
    let stmt = client.prepare(stmt).await?;

    let results = client
        .query(&stmt, &[&input])
        .await?
        .iter()
        .map(|row| RelatedConcept::from_row(row.clone()).unwrap())
        .collect::<Vec<RelatedConcept>>();

    Ok(results)
}

pub async fn get_concept_name_by_string(
    client: &Client,
    input: String,
) -> Result<Vec<String>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = include_str!("../sql/select_concept_for_non_numeric_input.sql");
    let stmt = client.prepare(stmt).await?;

    let results = client
        .query(&stmt, &[&input])
        .await?
        .iter()
        .map(|row| row.get("concept_name"))
        .collect::<Vec<String>>();

    Ok(results)
}

pub async fn get_descendant_concepts(
    client: &Client,
    concept_id: i32,
) -> Result<Vec<i32>, PgError> {
    info!("Getting descendant concepts for {}", &concept_id);
    let stmt = include_str!("../sql/select_descendant_concepts.sql");
    let stmt = client.prepare(stmt).await?;

    let results = client
        .query(&stmt, &[&concept_id])
        .await?
        .iter()
        .map(|row| row.get("concept_id"))
        .collect::<Vec<i32>>();

    Ok(results)
}

pub async fn get_batch_descendant_concepts(
    client: &Client,
    concept_ids: &[i32],
) -> Result<std::collections::HashMap<i32, Vec<i32>>, PgError> {
    use std::collections::HashMap;

    if concept_ids.is_empty() {
        return Ok(HashMap::new());
    }

    info!(
        "Getting descendant concepts for {} concepts",
        concept_ids.len()
    );

    // Build the SQL query with IN clause
    let placeholders: Vec<String> = (1..=concept_ids.len()).map(|i| format!("${}", i)).collect();
    let sql = format!(
        "SELECT ancestor_concept_id, descendant_concept_id as concept_id
         FROM cdm.concept_ancestor 
         WHERE ancestor_concept_id IN ({})
           AND min_levels_of_separation > 0",
        placeholders.join(", ")
    );

    let stmt = client.prepare(&sql).await?;

    // Convert concept_ids to references for the query
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = concept_ids
        .iter()
        .map(|id| id as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows = client.query(&stmt, &params).await?;

    // Group results by ancestor concept ID
    let mut result: HashMap<i32, Vec<i32>> = HashMap::new();

    // Initialize empty vectors for all requested concept IDs
    for &concept_id in concept_ids {
        result.insert(concept_id, Vec::new());
    }

    // Populate with actual descendants
    for row in rows {
        let ancestor_id: i32 = row.get("ancestor_concept_id");
        let descendant_id: i32 = row.get("concept_id");

        result
            .entry(ancestor_id)
            .or_insert_with(Vec::new)
            .push(descendant_id);
    }

    Ok(result)
}

pub async fn get_batch_mapped_concepts(
    client: &Client,
    concept_ids: &[i32],
) -> Result<std::collections::HashMap<i32, Vec<i32>>, PgError> {
    use std::collections::HashMap;

    if concept_ids.is_empty() {
        return Ok(HashMap::new());
    }

    info!("Getting mapped concepts for {} concepts", concept_ids.len());

    // Build the SQL query with IN clause
    let placeholders: Vec<String> = (1..=concept_ids.len()).map(|i| format!("${}", i)).collect();
    let sql = format!(
        "SELECT cr.concept_id_2 as source_concept_id, cr.concept_id_1 as mapped_concept_id
         FROM cdm.concept_relationship cr
         WHERE cr.concept_id_2 IN ({})
           AND cr.relationship_id = 'Maps to' 
           AND cr.invalid_reason IS NULL",
        placeholders.join(", ")
    );

    let stmt = client.prepare(&sql).await?;

    // Convert concept_ids to references for the query
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = concept_ids
        .iter()
        .map(|id| id as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows = client.query(&stmt, &params).await?;

    // Group results by source concept ID
    let mut result: HashMap<i32, Vec<i32>> = HashMap::new();

    // Initialize empty vectors for all requested concept IDs
    for &concept_id in concept_ids {
        result.insert(concept_id, Vec::new());
    }

    // Populate with actual mapped concepts
    for row in rows {
        let source_id: i32 = row.get("source_concept_id");
        let mapped_id: i32 = row.get("mapped_concept_id");

        result
            .entry(source_id)
            .or_insert_with(Vec::new)
            .push(mapped_id);
    }

    Ok(result)
}
