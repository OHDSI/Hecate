use anyhow::{Context, Result};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::embeddings::{CreateEmbeddingRequestArgs, Embedding};
use log::info;

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
