use futures::StreamExt;
use reqwest;
use sha2::{Digest, Sha256};

pub async fn get_size_and_hash(
    client: &reqwest::Client,
    url: &str,
) -> Result<(i64, String), reqwest::Error> {
    let response = client.get(url).send().await?;
    let mut stream = response.bytes_stream();
    let mut hasher = Sha256::new();
    let mut total_size = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        total_size += chunk.len() as i64;
        hasher.update(&chunk);
    }

    let hash = format!("{:x}", hasher.finalize());
    Ok((total_size, hash))
}
