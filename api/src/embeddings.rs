use async_openai::Client;
use async_openai::types::{CreateEmbeddingRequestArgs, Embedding};
use log::info;
use std::error::Error;

pub async fn fetch_embeddings(input: String) -> Result<Embedding, Box<dyn Error>> {
    info!("Fetching embedding from OpenAI for {:?}", &input);
    let client = Client::new();

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-large")
        .input(input.to_string())
        .dimensions(1024u32)
        .build()?;

    let response = client.embeddings().create(request).await?;
    let embedding = response.data[0].clone();
    Ok(embedding)
}
