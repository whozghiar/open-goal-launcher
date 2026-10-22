//! What the launcher gives Jak Level Editor (level-editor/, its plugin): the default library
//! (`<install>/jak-level-editor`), the disc the launcher already extracted, and ogle-extract
//! downloaded from the editor's release of its version (checked against GitHub's SHA-256 of it).
//! The editor runs no game: its exports are files to copy into a jak-project.

use std::{
  future::Future,
  path::{Path, PathBuf},
  sync::Arc,
};

use anyhow::Context;
use tauri::AppHandle;
use tauri_plugin_jak_level_editor::{
  EDITOR_VERSION, ExtractorProvider, Host, HostEvent, HostEvents,
};
use tracing::instrument;

use crate::{
  commands::CommandError,
  config::LauncherConfig,
  util::{file::delete_dir, network::download_release_asset, tar::extract_and_delete_archive},
};

const EXTRACTOR_URL: &str = "https://github.com/whozghiar/open-goal-level-builder/releases/download/v{version}/ogle-extract-{platform}.zip";

fn exe(dir: &Path, name: &str) -> PathBuf {
  dir.join(if cfg!(windows) {
    format!("{name}.exe")
  } else {
    name.to_owned()
  })
}

fn platform() -> Option<&'static str> {
  match (std::env::consts::OS, std::env::consts::ARCH) {
    ("windows", "x86_64") => Some("windows-x64"),
    ("linux", "x86_64") => Some("linux-x64"),
    ("macos", "x86_64") => Some("macos-x64"),
    ("macos", "aarch64") => Some("macos-arm64"),
    _ => None,
  }
}

/// ogle-extract's archive: the editor's release, or a local file for tests (JLE_EXTRACTOR_URL)
fn extractor_source() -> Option<String> {
  std::env::var("JLE_EXTRACTOR_URL").ok().or_else(|| {
    platform().map(|p| {
      EXTRACTOR_URL
        .replace("{version}", EDITOR_VERSION)
        .replace("{platform}", p)
    })
  })
}

fn extractor_dir(install: &Path) -> PathBuf {
  install
    .join("jak-level-editor")
    .join("tools")
    .join(format!("v{EDITOR_VERSION}"))
}

/// The game's files the launcher extracted (read only)
fn launcher_iso_data(install: &Path) -> PathBuf {
  install.join("active/jak3/data/iso_data/jak3")
}

/// Calls `progress(done, total)` once per percent (per MB when the total is unknown).
fn throttled(mut progress: impl FnMut(u64, u64)) -> impl FnMut(u64, u64) {
  let mut last = u64::MAX;
  move |done, total| {
    let step = if total > 0 {
      done * 100 / total
    } else {
      done >> 20
    };
    if step != last {
      last = step;
      progress(done, total);
    }
  }
}

/// A release's archive (checked against GitHub's SHA-256 of it; a local file for tests) unpacked
/// into `dest`.
async fn unpack(
  source: &str,
  dest: &Path,
  strip_top_dir: bool,
  progress: impl FnMut(u64, u64),
) -> anyhow::Result<()> {
  let name = source
    .rsplit(['/', '\\'])
    .next()
    .filter(|s| !s.is_empty())
    .with_context(|| format!("No archive name in {source}"))?;
  let archive = dest.join(name);
  std::fs::create_dir_all(dest)?;
  if source.starts_with("https://") {
    download_release_asset(source, &archive, throttled(progress)).await?;
  } else {
    std::fs::copy(source, &archive).with_context(|| format!("Unable to read {source}"))?;
  }
  extract_and_delete_archive(&archive, dest, strip_top_dir)
}

/// Runs a future of the launcher on its async runtime and waits for it (the editor calls its host
/// from a worker thread).
fn wait<T: Send + 'static>(
  future: impl Future<Output = Result<T, CommandError>> + Send + 'static,
) -> Result<T, String> {
  let (tx, rx) = std::sync::mpsc::channel();
  tauri::async_runtime::spawn(async move {
    let _ = tx.send(future.await.map_err(|e| format!("{e:#}")));
  });
  rx.recv().map_err(|e| e.to_string())?
}

struct Extractor {
  install: PathBuf,
}

impl ExtractorProvider for Extractor {
  fn fetch(&self, events: HostEvents) -> Result<PathBuf, String> {
    let dir = extractor_dir(&self.install);
    let path = exe(&dir, "ogle-extract");
    if path.is_file() {
      return Ok(path);
    }
    let source = extractor_source().ok_or("No ogle-extract for this system")?;
    let part = dir.with_file_name(format!("v{EDITOR_VERSION}.part"));
    let target = dir.clone();
    wait(async move {
      delete_dir(&part)?;
      unpack(&source, &part, false, move |done, total| {
        events(HostEvent::Progress { done, total })
      })
      .await?;
      delete_dir(&target)?;
      std::fs::rename(&part, &target)?;
      Ok(())
    })?;
    if path.is_file() {
      Ok(path)
    } else {
      Err("No ogle-extract in the editor's release".into())
    }
  }
}

/// The host the editor gets from the launcher's settings (its installation folder).
pub fn host(config: &LauncherConfig) -> Host {
  let Some(install) = config.installation_dir.clone() else {
    return Host {
      name: "launcher".into(),
      ..Default::default()
    };
  };
  let extractor = exe(&extractor_dir(&install), "ogle-extract");
  let iso = launcher_iso_data(&install);
  Host {
    name: "launcher".into(),
    default_library: Some(install.join("jak-level-editor")),
    extract_source: iso.is_dir().then_some(iso),
    extractor: extractor.is_file().then_some(extractor),
    extractor_provider: Some(Arc::new(Extractor { install })),
  }
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn refresh_level_editor_host(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  let host = host(&*config.lock().await);
  tauri_plugin_jak_level_editor::set_host(&app_handle, host);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  #[test]
  fn the_extractor_comes_from_its_archive() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("ogle-extract-test.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    let name = exe(Path::new(""), "ogle-extract");
    zip.start_file(name.to_string_lossy(), options).unwrap();
    zip.write_all(b"exe").unwrap();
    zip
      .start_file("ogle-extract-data/game.json", options)
      .unwrap();
    zip.write_all(b"{}").unwrap();
    zip.finish().unwrap();
    unsafe { std::env::set_var("JLE_EXTRACTOR_URL", &archive) };

    let install = tmp.path().join("install");
    let provider = Extractor {
      install: install.clone(),
    };
    let path = provider.fetch(Arc::new(|_| {})).unwrap();
    let dir = extractor_dir(&install);
    assert_eq!(path, dir.join(&name));
    assert!(dir.join("ogle-extract-data/game.json").is_file());
    assert!(!dir.join("ogle-extract-test.zip").exists());
    assert_eq!(provider.fetch(Arc::new(|_| {})).unwrap(), path);
  }
}
