//! Jak Animation Studio (whozghiar/jak-animation-studio), a program of its own the
//! launcher installs, updates and starts as it does Jak Level Editor: the portable build of each
//! version in `<install>/versions/animation-studio/<version>/`, the active one in the settings. The
//! studio gets the launcher's language, its default creations folder (`<install>/animation-studio`),
//! the games installed with the active version (`--game-install`) and that version
//! (`--tooling-version`, empty without one): it tries models in those games only, with the active gk
//! and goalc (`<install>/active/bin`).

use std::path::{Path, PathBuf};

use anyhow::Context;
use tracing::instrument;

use super::level_editor::{exe, games_to_try, launch_args, unpack};
use crate::{
  commands::CommandError,
  config::LauncherConfig,
  util::{file::delete_dir, process::create_std_log_file},
};

/// The studio started last, so the button does not open a second one.
#[derive(Default)]
pub struct AnimationStudioProcess(tokio::sync::Mutex<Option<std::process::Child>>);

fn versions_dir(install: &Path) -> PathBuf {
  install.join("versions").join("animation-studio")
}

fn studio_args(config: &LauncherConfig, install: &Path) -> Vec<String> {
  let mut args = launch_args(
    &config.locale,
    &install.join("animation-studio"),
    install,
    &games_to_try(config),
  );
  args.push(format!(
    "--tooling-version={}",
    config.active_version.as_deref().unwrap_or_default()
  ));
  args
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn download_animation_studio_version(
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
pub async fn remove_animation_studio_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
) -> Result<(), CommandError> {
  let mut config = config.lock().await;
  delete_dir(versions_dir(&config.install_dir()?).join(&version))?;
  if config.animation_studio_version.as_deref() == Some(version.as_str()) {
    config.set_animation_studio_version(None)?;
  }
  Ok(())
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn set_animation_studio_version(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  version: String,
) -> Result<(), CommandError> {
  config
    .lock()
    .await
    .set_animation_studio_version(Some(version))?;
  Ok(())
}

/// Starts the active version of the studio; false when the one started before is still open.
#[instrument(skip(config, process, app_handle))]
#[tauri::command]
pub async fn launch_animation_studio(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  process: tauri::State<'_, AnimationStudioProcess>,
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
  let version = config.animation_studio_version.as_deref().ok_or_else(|| {
    CommandError::Configuration("No Animation Studio version is active".to_owned())
  })?;
  let dir = versions_dir(&install).join(version);
  let log_file = create_std_log_file(&app_handle, "animation-studio.log".to_owned(), false)?;

  let child = std::process::Command::new(exe(&dir, "jak-animation-studio"))
    .args(studio_args(&config, &install))
    .current_dir(&dir)
    .stdout(log_file.try_clone()?)
    .stderr(log_file)
    .spawn()
    .with_context(|| format!("Unable to start the Animation Studio {version}"))?;
  *running = Some(child);
  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::SupportedGame;

  #[test]
  fn the_studio_tries_in_the_games_of_the_active_tooling_only() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path();
    let mut config: LauncherConfig = serde_json::from_str("{}").unwrap();
    config.locale = "fr-FR".to_owned();
    config.active_version = Some("v0.3.9".to_owned());
    for (game, version) in [
      (SupportedGame::Jak3, "v0.3.9"),
      (SupportedGame::Jak2, "v0.3.9-whozghiar.1"),
    ] {
      config
        .get_supported_game_config_mut(game)
        .set_installed(true)
        .set_version(Some(version.to_owned()));
    }

    let args = studio_args(&config, install);
    assert_eq!(args[..2], ["--lang", "fr"]);
    assert!(args.contains(&format!(
      "--default-library={}",
      install.join("animation-studio").display()
    )));
    assert!(args.contains(&format!("--game-install=jak3={}", install.display())));
    // Jak II was built by another tooling: the active gk and goalc do not play it
    assert!(!args.iter().any(|a| a.starts_with("--game-install=jak2")));
    assert_eq!(args.last().unwrap(), "--tooling-version=v0.3.9");

    config.active_version = None;
    let args = studio_args(&config, install);
    assert!(!args.iter().any(|a| a.starts_with("--game-install")));
    assert_eq!(args.last().unwrap(), "--tooling-version=");
  }
}
