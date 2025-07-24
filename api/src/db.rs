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
