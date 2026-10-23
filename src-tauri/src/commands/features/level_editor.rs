//! Jak Level Editor (whozghiar/jak-level-editor), a program of its own the launcher installs and
//! updates as it does the tooling: the portable build of each version downloaded from its release
//! into `<install>/versions/level-editor/<version>/` (checked against GitHub's SHA-256 of it), the
//! active one in the settings. The launcher starts it with what it knows: its preferences in the
//! launcher's config folder, its default library (`<install>/jak-level-editor`), the discs already
//! extracted. The editor runs no game: its exports are files to copy into a jak-project.

use std::path::{Path, PathBuf};

use anyhow::Context;
use tauri::Manager;
use tracing::instrument;

use crate::{
  commands::CommandError,
  config::LauncherConfig,
  util::{
    file::delete_dir, network::download_release_asset, process::create_std_log_file,
    tar::extract_and_delete_archive,
  },
};

/// The editor started last, so the button does not open a second one on the same preferences.
#[derive(Default)]
pub struct LevelEditorProcess(tokio::sync::Mutex<Option<std::process::Child>>);

fn exe(dir: &Path, name: &str) -> PathBuf {
  dir.join(if cfg!(windows) {
    format!("{name}.exe")
  } else {
    name.to_owned()
  })
}

fn versions_dir(install: &Path) -> PathBuf {
  install.join("versions").join("level-editor")
}

/// The editor's command line. Its own options go as `--name=value`, which an editor that does not
/// know them yet skips whole (v1.0.0 would take a separate value for a project to open).
fn editor_args(locale: &str, install: &Path) -> Vec<String> {
  let mut args = Vec::new();
  if let Some(lang) = locale.split('-').next().filter(|l| !l.is_empty()) {
    args.extend(["--lang".to_owned(), lang.to_owned()]);
  }
  args.push(format!(
    "--default-library={}",
    install.join("jak-level-editor").display()
  ));
  for game in ["jak1", "jak2", "jak3"] {
    let disc = install
      .join("active")
      .join(game)
      .join("data")
      .join("iso_data")
      .join(game);
    if disc.is_dir() {
      args.push(format!("--extract-source={game}={}", disc.display()));
    }
  }
  args
}

/// A release's archive (checked against GitHub's SHA-256 of it; a local file in the tests) unpacked
/// into `dest`.
async fn unpack(source: &str, dest: &Path) -> anyhow::Result<()> {
  let name = source
    .rsplit(['/', '\\'])
    .next()
    .filter(|s| !s.is_empty())
    .with_context(|| format!("No archive name in {source}"))?;
  let archive = dest.join(name);
  delete_dir(dest)?;
  std::fs::create_dir_all(dest)?;
  if source.starts_with("https://") {
    download_release_asset(source, &archive, |_, _| {}).await?;
  } else {
    std::fs::copy(source, &archive).with_context(|| format!("Unable to read {source}"))?;
  }
  extract_and_delete_archive(&archive, dest, false)
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn download_level_editor_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
  url: String,
) -> Result<(), CommandError> {
  let dest = versions_dir(&config.lock().await.install_dir()?).join(&version);
  unpack(&url, &dest).await?;
  Ok(())
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn remove_level_editor_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
) -> Result<(), CommandError> {
  let mut config = config.lock().await;
  delete_dir(versions_dir(&config.install_dir()?).join(&version))?;
  if config.level_editor_version.as_deref() == Some(version.as_str()) {
    config.set_level_editor_version(None)?;
  }
  Ok(())
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn set_level_editor_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
) -> Result<(), CommandError> {
  config
    .lock()
    .await
    .set_level_editor_version(Some(version))?;
  Ok(())
}

/// Starts the active version of the editor; false when the one started before is still open.
#[instrument(skip(config, process, app_handle))]
#[tauri::command]
pub async fn launch_level_editor(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  process: tauri::State<'_, LevelEditorProcess>,
  app_handle: tauri::AppHandle,
) -> Result<bool, CommandError> {
  let mut running = process.0.lock().await;
  if let Some(child) = running.as_mut()
    && child.try_wait()?.is_none()
  {
    return Ok(false);
  }

  let config = config.lock().await;
  let install = config.install_dir()?;
  let version = config
    .level_editor_version
    .as_deref()
    .ok_or_else(|| CommandError::Configuration("No Level Editor version is active".to_owned()))?;
  let dir = versions_dir(&install).join(version);
  // the preferences (library, caches) the editor had when the launcher hosted it
  let prefs = app_handle.path().app_config_dir()?.join("jak-level-editor");
  let log_file = create_std_log_file(&app_handle, "level-editor.log".to_owned(), false)?;

  let child = std::process::Command::new(exe(&dir, "jak-level-editor"))
    .args(editor_args(&config.locale, &install))
    .env("JLE_PREFS_DIR", prefs)
    .current_dir(&dir)
    .stdout(log_file.try_clone()?)
    .stderr(log_file)
    .spawn()
    .with_context(|| format!("Unable to start the Level Editor {version}"))?;
  *running = Some(child);
  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  #[tokio::test]
  async fn a_version_comes_from_its_portable_archive() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("jak-level-editor-portable-test.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    let name = exe(Path::new(""), "jak-level-editor");
    zip.start_file(name.to_string_lossy(), options).unwrap();
    zip.write_all(b"exe").unwrap();
    zip.start_file("lang/en.json", options).unwrap();
    zip.write_all(b"{}").unwrap();
    zip.finish().unwrap();

    let dest = versions_dir(&tmp.path().join("install")).join("v1.0.0");
    unpack(&archive.to_string_lossy(), &dest).await.unwrap();
    assert!(dest.join(&name).is_file());
    assert!(dest.join("lang/en.json").is_file());
    assert!(!dest.join("jak-level-editor-portable-test.zip").exists());
  }

  #[test]
  fn the_editor_gets_the_language_the_library_and_the_discs() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path();
    let jak3 = install
      .join("active")
      .join("jak3")
      .join("data")
      .join("iso_data")
      .join("jak3");
    std::fs::create_dir_all(&jak3).unwrap();

    let args = editor_args("fr-FR", install);
    assert_eq!(args[..2], ["--lang", "fr"]);
    assert!(args.contains(&format!(
      "--default-library={}",
      install.join("jak-level-editor").display()
    )));
    assert!(args.contains(&format!("--extract-source=jak3={}", jak3.display())));
    assert!(!args.iter().any(|a| a.starts_with("--extract-source=jak2")));
    assert!(!editor_args("", install).contains(&"--lang".to_owned()));
  }
}
