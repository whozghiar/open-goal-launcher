use anyhow::{Context, Result};
use std::path::Path;
use tokio;
use tokio::io::AsyncWriteExt;

pub async fn download_file(url: &str, destination: &Path) -> Result<()> {
  download_file_with_progress(url, destination, |_, _| {}).await
}

/// `on_progress(downloaded, total)`, total being 0 when the server does not tell it
pub async fn download_file_with_progress(
  url: &str,
  destination: &Path,
  mut on_progress: impl FnMut(u64, u64),
) -> Result<()> {
  if let Some(parent) = destination.parent() {
    tokio::fs::create_dir_all(parent).await?;
  }

  let mut response = reqwest::get(url)
    .await
    .with_context(|| format!("Failed to download file from: {url}"))?
    .error_for_status()
    .with_context(|| format!("Server returned error for {url}"))?;

  let mut file = tokio::fs::File::create(destination)
    .await
    .with_context(|| {
      format!(
        "Failed to create destination file {}",
        destination.display()
      )
    })?;

  let total = response.content_length().unwrap_or(0);
  let mut downloaded = 0;
  while let Some(chunk) = response.chunk().await? {
    file.write_all(&chunk).await?;
    downloaded += chunk.len() as u64;
    on_progress(downloaded, total);
  }

  Ok(())
}

pub async fn download_json(url: &str) -> Result<String> {
  let res = reqwest::get(url)
    .await
    .with_context(|| format!("Failed to download json from {url}"))?
    .error_for_status()
    .with_context(|| format!("Server returned error status for {url}"))?;

  res
    .text()
    .await
    .with_context(|| format!("Failed to read response body from {url}"))
}
