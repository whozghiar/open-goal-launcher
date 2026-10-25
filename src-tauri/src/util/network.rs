use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
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

/// The GitHub API's release of an asset's download URL
/// (`https://github.com/<owner>/<repo>/releases/download/<tag>/<name>`), and the asset's name.
fn release_api_url(url: &str) -> Option<(String, &str)> {
  let path = url.strip_prefix("https://github.com/")?;
  match path.split('/').collect::<Vec<_>>()[..] {
    [owner, repo, "releases", "download", tag, name] => Some((
      format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}"),
      name,
    )),
    _ => None,
  }
}

fn sha256_hex(path: &Path) -> Result<String> {
  let mut file = std::fs::File::open(path)?;
  let mut hasher = Sha256::new();
  std::io::copy(&mut file, &mut hasher)?;
  Ok(format!("{:x}", hasher.finalize()))
}

/// Downloads an asset of a GitHub release to `destination`, then checks it against the SHA-256
/// GitHub gives for it (its `digest`): a file that differs is deleted and refused. GitHub has no
/// digest for the assets uploaded before mid-2025 (most of open-goal/jak-project's releases): those
/// are downloaded unchecked, as the launcher always did.
pub async fn download_release_asset(
  url: &str,
  destination: &Path,
  on_progress: impl FnMut(u64, u64),
) -> Result<()> {
  let (api, name) =
    release_api_url(url).with_context(|| format!("Not a GitHub release asset: {url}"))?;
  let release: serde_json::Value = reqwest::Client::new()
    .get(&api)
    .header(reqwest::header::USER_AGENT, "OpenGOAL-Launcher")
    .send()
    .await
    .with_context(|| format!("Failed to read the release of {url}"))?
    .error_for_status()
    .with_context(|| format!("Server returned error status for {api}"))?
    .json()
    .await?;
  let expected = release["assets"]
    .as_array()
    .and_then(|assets| assets.iter().find(|a| a["name"] == name))
    .and_then(|a| a["digest"].as_str())
    .and_then(|d| d.strip_prefix("sha256:"))
    .map(str::to_ascii_lowercase);
  download_file_with_progress(url, destination, on_progress).await?;
  let Some(expected) = expected else {
    tracing::warn!("GitHub gives no SHA-256 for {name}: downloaded unchecked");
    return Ok(());
  };
  let actual = sha256_hex(destination)?;
  if actual != expected {
    let _ = std::fs::remove_file(destination);
    bail!("{name} is not the file of its release (SHA-256 {actual}, expected {expected}): deleted");
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_release_asset_and_its_digest() {
    assert_eq!(
      release_api_url(
        "https://github.com/whozghiar/jak-project/releases/download/v0.3.9-whozghiar.1/opengoal-windows-v0.3.9-whozghiar.1.zip"
      ),
      Some((
        "https://api.github.com/repos/whozghiar/jak-project/releases/tags/v0.3.9-whozghiar.1"
          .to_owned(),
        "opengoal-windows-v0.3.9-whozghiar.1.zip"
      ))
    );
    assert!(release_api_url("https://example.com/a.zip").is_none());
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("abc");
    std::fs::write(&file, "abc").unwrap();
    assert_eq!(
      sha256_hex(&file).unwrap(),
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
  }

  // the real release, on the network: cargo test -- --ignored a_real_release_asset_checked
  #[tokio::test]
  #[ignore]
  async fn a_real_release_asset_checked() {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("lsp.bin");
    download_release_asset(
      "https://github.com/whozghiar/jak-project/releases/download/v0.3.9-whozghiar.1/opengoal-lsp-linux-v0.3.9-whozghiar.1.bin",
      &dest,
      |_, _| {},
    )
    .await
    .unwrap();
    assert!(dest.is_file());
  }

  // an official release from before GitHub's digests: cargo test -- --ignored without_a_digest
  #[tokio::test]
  #[ignore]
  async fn a_real_release_asset_without_a_digest() {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("lsp.bin");
    download_release_asset(
      "https://github.com/open-goal/jak-project/releases/download/v0.2.25/opengoal-lsp-linux-v0.2.25.bin",
      &dest,
      |_, _| {},
    )
    .await
    .unwrap();
    assert!(dest.is_file());
  }
}
