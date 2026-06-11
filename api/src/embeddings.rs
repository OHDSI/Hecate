use anyhow::{Context, Result};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::embeddings::{CreateEmbeddingRequestArgs, Embedding};
use log::info;
use moka::future::Cache;

pub async fn fetch_embeddings_cached(
    client: &Client<OpenAIConfig>,
    input: &str,
    cache: &Cache<String, Vec<f32>>,
) -> Result<Vec<f32>> {
    if let Some(vector) = cache.get(input).await {
        return Ok(vector);
    }
    let vector = fetch_embeddings(client, input).await?.embedding;
    cache.insert(input.to_string(), vector.clone()).await;
    Ok(vector)
}

pub async fn fetch_embeddings(client: &Client<OpenAIConfig>, input: &str) -> Result<Embedding> {
    info!("Fetching embedding from OpenAI for {:?}", input);

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-large")
        .input(input)
        .dimensions(1024u32)
        .build()?;

    let response = client.embeddings().create(request).await?;
    let embedding = response
        .data
        .into_iter()
        .next()
        .context("OpenAI returned no embeddings")?;
    Ok(embedding)
}
