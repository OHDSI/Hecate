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
    cache
        .try_get_with(input.to_string(), async {
            fetch_embeddings(client, input)
                .await
                .map(|result| result.embedding)
        })
        .await
        .map_err(|error| anyhow::anyhow!("Embedding request failed: {error}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, HttpServer, web};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;

    #[actix_web::test]
    async fn concurrent_embedding_misses_share_a_fetch_and_failures_are_not_cached() {
        let requests = Arc::new(AtomicUsize::new(0));
        let server_requests = requests.clone();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(server_requests.clone()))
                .route("/embeddings", web::post().to(
                    |requests: web::Data<Arc<AtomicUsize>>, body: web::Json<serde_json::Value>| async move {
                        requests.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        let data = if body["input"] == "empty" {
                            serde_json::json!([])
                        } else {
                            serde_json::json!([{"object": "embedding", "index": 0, "embedding": [0.1, 0.2]}])
                        };
                        HttpResponse::Ok().json(serde_json::json!({
                            "object": "list", "data": data, "model": "text-embedding-3-large",
                            "usage": {"prompt_tokens": 1, "total_tokens": 1}
                        }))
                    }
                ))
        }).workers(1).listen(listener).unwrap().run();
        let handle = server.handle();
        let server_task = actix_web::rt::spawn(server);
        let config = OpenAIConfig::new()
            .with_api_base(format!("http://{address}"))
            .with_api_key("test-key");
        let client = Client::with_config(config).with_http_client(
            reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        );
        let cache = Cache::new(10);
        let (first, second) = tokio::join!(
            fetch_embeddings_cached(&client, "test", &cache),
            fetch_embeddings_cached(&client, "test", &cache),
        );
        assert_eq!(first.unwrap(), vec![0.1, 0.2]);
        assert_eq!(second.unwrap(), vec![0.1, 0.2]);
        assert_eq!(
            fetch_embeddings_cached(&client, "test", &cache)
                .await
                .unwrap(),
            vec![0.1, 0.2]
        );
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        // A failed fetch must be retried on the next request rather than cached.
        for _ in 0..2 {
            assert!(
                fetch_embeddings_cached(&client, "empty", &cache)
                    .await
                    .is_err()
            );
        }
        assert_eq!(requests.load(Ordering::SeqCst), 3);
        handle.stop(true).await;
        server_task.await.unwrap().unwrap();
    }
}
