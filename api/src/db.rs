use crate::domain::{Concept, RelatedConcept};
use crate::errors::PgError;
use deadpool_postgres::Client;
use log::info;
use moka::future::Cache;
use std::sync::LazyLock;
use tokio_pg_mapper::FromTokioPostgresRow;

static VOCAB_SCHEMA: LazyLock<String> =
    LazyLock::new(|| std::env::var("VOCAB_SCHEMA").expect("VOCAB_SCHEMA env variable must be set"));

fn sql(template: &str) -> String {
    template.replace("{VOCAB_SCHEMA}", &VOCAB_SCHEMA)
}

pub async fn get_concept_name_by_number(
    client: &Client,
    input: i32,
) -> Result<Vec<String>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = sql(include_str!("../sql/select_concept_for_numeric_input.sql"));
    let stmt = client.prepare(&stmt).await?;

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
    let stmt = sql(include_str!("../sql/select_concept_by_id.sql"));
    let stmt = client.prepare(&stmt).await?;

    let rows = client.query(&stmt, &[&input]).await?;
    let row = rows.first().ok_or(PgError::NotFound)?;
    let result = Concept::from_row(row.clone())?;

    Ok(result)
}

pub async fn get_concept_relationships(
    client: &Client,
    input: i32,
) -> Result<Vec<RelatedConcept>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = sql(include_str!("../sql/select_related_concepts.sql"));
    let stmt = client.prepare(&stmt).await?;

    let rows = client.query(&stmt, &[&input]).await?;
    let results = rows
        .iter()
        .map(|row| RelatedConcept::from_row(row.clone()))
        .collect::<Result<Vec<RelatedConcept>, _>>()?;

    Ok(results)
}

pub async fn get_concept_phoebe(
    client: &Client,
    input: i32,
) -> Result<Vec<RelatedConcept>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = sql(include_str!("../sql/select_phoebe_concepts.sql"));
    let stmt = client.prepare(&stmt).await?;

    let rows = client.query(&stmt, &[&input]).await?;
    let results = rows
        .iter()
        .map(|row| RelatedConcept::from_row(row.clone()))
        .collect::<Result<Vec<RelatedConcept>, _>>()?;

    Ok(results)
}

pub async fn get_concept_name_by_string(
    client: &Client,
    input: String,
) -> Result<Vec<String>, PgError> {
    info!("Checking vocabulary for {}", &input.to_string());
    let stmt = sql(include_str!(
        "../sql/select_concept_for_non_numeric_input.sql"
    ));
    let stmt = client.prepare(&stmt).await?;

    let results = client
        .query(&stmt, &[&input])
        .await?
        .iter()
        .map(|row| row.get("concept_name"))
        .collect::<Vec<String>>();

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
         FROM {schema}.concept_ancestor
         WHERE ancestor_concept_id IN ({placeholders})
           AND min_levels_of_separation > 0",
        schema = *VOCAB_SCHEMA,
        placeholders = placeholders.join(", ")
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

        result.entry(ancestor_id).or_default().push(descendant_id);
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
         FROM {schema}.concept_relationship cr
         WHERE cr.concept_id_2 IN ({placeholders})
           AND cr.relationship_id = 'Maps to'
           AND cr.invalid_reason IS NULL",
        schema = *VOCAB_SCHEMA,
        placeholders = placeholders.join(", ")
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

        result.entry(source_id).or_default().push(mapped_id);
    }

    Ok(result)
}

pub async fn get_concept_children_cached(
    client: &Client,
    cache: &Cache<i32, Vec<Concept>>,
    concept_id: i32,
) -> Result<Vec<Concept>, PgError> {
    // Check cache first
    if let Some(cached_children) = cache.get(&concept_id).await {
        return Ok(cached_children);
    }

    // Get from database
    let children = get_concept_descendants(client, concept_id, 1).await?;

    // Cache the result
    cache.insert(concept_id, children.clone()).await;

    Ok(children)
}

pub async fn get_concept_parents_cached(
    client: &Client,
    cache: &Cache<i32, Vec<Concept>>,
    concept_id: i32,
) -> Result<Vec<Concept>, PgError> {
    // Check cache first
    if let Some(cached_parents) = cache.get(&concept_id).await {
        return Ok(cached_parents);
    }

    // Get from database
    let parents = get_concept_ancestors(client, concept_id, 1).await?;

    // Cache the result
    cache.insert(concept_id, parents.clone()).await;

    Ok(parents)
}

async fn get_concept_descendants(
    client: &Client,
    concept_id: i32,
    levels_of_separation: i32,
) -> Result<Vec<Concept>, PgError> {
    let stmt = sql(include_str!("../sql/select_concept_descendants.sql"));
    let stmt = client.prepare(&stmt).await?;

    let results = client
        .query(&stmt, &[&concept_id, &levels_of_separation])
        .await?
        .iter()
        .map(|row| Concept::from_row(row.clone()))
        .collect::<Result<Vec<Concept>, _>>()?;

    Ok(results)
}

async fn get_concept_ancestors(
    client: &Client,
    concept_id: i32,
    levels_of_separation: i32,
) -> Result<Vec<Concept>, PgError> {
    let stmt = sql(include_str!("../sql/select_concept_ancestors.sql"));
    let stmt = client.prepare(&stmt).await?;

    let results = client
        .query(&stmt, &[&concept_id, &levels_of_separation])
        .await?
        .iter()
        .map(|row| Concept::from_row(row.clone()))
        .collect::<Result<Vec<Concept>, _>>()?;

    Ok(results)
}

pub async fn map_to_standard(client: &Client, concept_id: i32) -> Result<Vec<Concept>, PgError> {
    let stmt = sql(include_str!("../sql/select_concepts_by_relation.sql"));
    let stmt = client.prepare(&stmt).await?;

    let results = client
        .query(&stmt, &[&concept_id, &"Maps to"])
        .await?
        .iter()
        .map(|row| Concept::from_row(row.clone()))
        .collect::<Result<Vec<Concept>, _>>()?
        .into_iter()
        .filter(|concept| {
            concept
                .standard_concept
                .as_ref()
                .is_some_and(|sc| sc.eq_ignore_ascii_case("S"))
        })
        .collect::<Vec<Concept>>();

    Ok(results)
}
