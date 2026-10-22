//! What the launcher gives Jak Level Editor (level-editor/, its plugin): the default library, the
//! disc already extracted, ogle-extract downloaded from the editor's release, and the game runtime:
//! the isolated game (game.rs) where the edited levels are played with the programs of the active
//! tooling version, a release of whozghiar/jak-project. The level's installation is the editor's
//! (`jle_core::io::level_package`); the launcher prepares the game, builds it and starts it, as one
//! of its jobs.

pub mod game;

use std::{
  future::Future,
  path::{Path, PathBuf},
  process::Stdio,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::Context;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_jak_level_editor::{
  EDITOR_VERSION, ExtractorProvider, GameRuntime, Host, HostEvent, HostEvents, LevelPackage,
  PlayOutcome, RuntimeInfo, jle_core::i18n::tr,
};
use tracing::instrument;

use crate::{
  commands::{CommandError, binaries::format_exit_code},
  config::LauncherConfig,
  util::{
    file::delete_dir,
    network::download_release_asset,
    process::{create_log_file, watch_process},
    tar::extract_and_delete_archive,
  },
};
use game::{EditedGame, GAME, RuntimeState, exe, launcher_iso_data, stop_game};

const EXTRACTOR_URL: &str = "https://github.com/whozghiar/open-goal-level-builder/releases/download/v{version}/ogle-extract-{platform}.zip";

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

fn log_line(app: &AppHandle, line: String) {
  let _ = app.emit("log_update", serde_json::json!({ "logs": line }));
}

fn step(events: &HostEvents, key: &str, args: &[&str]) {
  events(HostEvent::Step {
    key: key.into(),
    args: args.iter().map(|a| a.to_string()).collect(),
  });
}

/// The game made from its tooling version when it was made from another one (its data copied
/// again, its levels installed again); it then still has to be decompiled.
async fn prepare(app: &AppHandle, game: EditedGame) -> Result<RuntimeState, CommandError> {
  if let Some(problem) = game.problem() {
    return Err(CommandError::GameFeatures(tr(problem).into()));
  }
  if game.stale() {
    log_line(
      app,
      tr("runtime.step.prepare").replacen("%s", &game.version, 1),
    );
  }
  tauri::async_runtime::spawn_blocking(move || game.prepare())
    .await
    .map_err(|e| CommandError::GameFeatures(e.to_string()))?
    .map_err(CommandError::GameFeatures)
}

/// A program of the tooling, its output in the job's logs and the launcher's log folder.
async fn run(app: &AppHandle, command: std::process::Command) -> Result<(), CommandError> {
  let mut command = tokio::process::Command::from(command);
  command.stdout(Stdio::piped()).stderr(Stdio::piped());
  #[cfg(windows)]
  {
    command.creation_flags(0x08000000);
  }
  tracing::info!("Running {:?}", command.as_std());
  let mut child = command
    .spawn()
    .context("Failed to spawn the game's tools")?;
  let mut log_file =
    create_log_file(app, format!("extractor-{GAME}-level_edited.log"), true).await?;
  let status = watch_process(&mut log_file, &mut child, app).await?;
  if status.success() {
    return Ok(());
  }
  Err(CommandError::GameFeatures(
    status
      .code()
      .map(format_exit_code)
      .map(|code| format!("Unexpected error occurred with code {code}"))
      .unwrap_or_else(|| "Unexpected error occurred".to_owned()),
  ))
}

/// Prepared, then the disc decompiled into it once per version.
async fn decompile(app: &AppHandle, game: EditedGame) -> Result<(), CommandError> {
  let mut state = prepare(app, game.clone()).await?;
  if state.decompiled {
    return Ok(());
  }
  run(app, game.decompile_command()).await?;
  state.decompiled = true;
  state
    .write(&game.folder)
    .map_err(CommandError::GameFeatures)
}

async fn compile(app: &AppHandle, game: &EditedGame) -> Result<(), CommandError> {
  // the game started before keeps open the files the build writes
  stop_game();
  run(app, game.compile_command()).await
}

fn install_level(game: &EditedGame, level_dir: &Path) -> Result<(), CommandError> {
  let package = LevelPackage::read(level_dir).map_err(CommandError::GameFeatures)?;
  if let Some(problem) = game.problem() {
    return Err(CommandError::GameFeatures(tr(problem).into()));
  }
  game
    .install_level(&package)
    .map_err(CommandError::GameFeatures)
}

async fn launch(game: EditedGame, name: String) -> Result<(), CommandError> {
  tauri::async_runtime::spawn_blocking(move || game.launch(&name))
    .await
    .map_err(|e| CommandError::GameFeatures(e.to_string()))?
    .map_err(CommandError::GameFeatures)
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

struct LevelEdited {
  app: AppHandle,
  install: PathBuf,
  /// the active tooling version (the host is given again when it changes)
  version: String,
  /// info() is asked at each of the editor's get_state: kept a moment
  info: Mutex<Option<(Instant, RuntimeInfo)>>,
}

impl LevelEdited {
  fn game(&self) -> EditedGame {
    EditedGame::new(&self.install, &self.version)
  }
}

impl GameRuntime for LevelEdited {
  fn info(&self) -> RuntimeInfo {
    let mut kept = self.info.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, info)) = kept.as_ref()
      && at.elapsed() < Duration::from_secs(2)
    {
      return info.clone();
    }
    let game = self.game();
    let problem = game.problem();
    let info = RuntimeInfo {
      kind: "launcher".into(),
      label: "OpenGOAL Launcher".into(),
      available: problem.is_none(),
      reason: problem.unwrap_or_default().into(),
      games: vec![GAME.into()],
      installed: game.installed(),
    };
    *kept = Some((Instant::now(), info.clone()));
    info
  }

  fn install_level(&self, package: &LevelPackage, events: HostEvents) -> Result<(), String> {
    let (app, game, dir) = (self.app.clone(), self.game(), package.dir.clone());
    step(&events, "runtime.step.install", &[&package.manifest.name]);
    wait(async move {
      decompile(&app, game.clone()).await?;
      install_level(&game, &dir)?;
      step(&events, "runtime.step.compile", &[]);
      compile(&app, &game).await
    })
  }

  fn remove_level(&self, _: &str, name: &str, events: HostEvents) -> Result<(), String> {
    let (app, game, name) = (self.app.clone(), self.game(), name.to_owned());
    step(&events, "runtime.step.remove", &[&name]);
    wait(async move {
      game
        .remove_level(&name)
        .map_err(CommandError::GameFeatures)?;
      decompile(&app, game.clone()).await?;
      step(&events, "runtime.step.compile", &[]);
      compile(&app, &game).await
    })
  }

  fn launch(&self, _: &str, name: &str, events: HostEvents) -> Result<(), String> {
    let (app, game, name) = (self.app.clone(), self.game(), name.to_owned());
    wait(async move {
      // made again for the active version, then built, before it starts
      if game.stale() {
        decompile(&app, game.clone()).await?;
        step(&events, "runtime.step.compile", &[]);
        compile(&app, &game).await?;
      }
      step(&events, "runtime.step.launch", &[&name]);
      launch(game, name).await
    })
  }

  /// The launcher's main window runs it as one of its jobs (job type `playEditedLevel`).
  fn play(&self, package: &LevelPackage, events: HostEvents) -> Result<PlayOutcome, String> {
    let Some(main) = self.app.get_webview_window("main") else {
      self.install_level(package, events.clone())?;
      self.launch(GAME, &package.manifest.name, events)?;
      return Ok(PlayOutcome::Started);
    };
    let _ = main.unminimize();
    let _ = main.set_focus();
    self
      .app
      .emit_to(
        "main",
        "level_edited:play",
        serde_json::json!({
          "levelDir": package.dir,
          "levelName": package.manifest.name,
        }),
      )
      .map_err(|e| e.to_string())?;
    Ok(PlayOutcome::HandedOver)
  }
}

/// The host the editor gets from the launcher's settings (its installation folder, its active
/// tooling version).
pub fn host(app: &AppHandle, config: &LauncherConfig) -> Host {
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
    extractor_provider: Some(Arc::new(Extractor {
      install: install.clone(),
    })),
    runtime: Some(Arc::new(LevelEdited {
      app: app.clone(),
      install,
      version: config.active_version.clone().unwrap_or_default(),
      info: Mutex::default(),
    })),
  }
}

/// The isolated game of the active tooling version.
async fn active_game(
  config: &tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<EditedGame, CommandError> {
  let config = config.lock().await;
  Ok(EditedGame::new(
    &config.install_dir()?,
    config.active_version.as_deref().unwrap_or_default(),
  ))
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn refresh_level_editor_host(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  let host = host(&app_handle, &*config.lock().await);
  tauri_plugin_jak_level_editor::set_host(&app_handle, host);
  Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEditedInfo {
  dir: PathBuf,
  levels: Vec<String>,
  /// the tooling version it was made with ("" before its first Play)
  version: String,
  active_version: String,
  /// made again for the active version at the next Play
  stale: bool,
  /// why it cannot be played with the active version
  problem: Option<String>,
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_info(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<LevelEditedInfo, CommandError> {
  let game = active_game(&config).await?;
  Ok(LevelEditedInfo {
    levels: game.installed(),
    version: RuntimeState::read(&game.folder).version,
    active_version: game.version.clone(),
    stale: game.stale(),
    problem: game.problem().map(|p| tr(p).to_string()),
    dir: game.folder,
  })
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_prepare_runtime(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  prepare(&app_handle, active_game(&config).await?).await?;
  Ok(())
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_decompile(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  decompile(&app_handle, active_game(&config).await?).await
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_install_level(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  level_dir: PathBuf,
) -> Result<(), CommandError> {
  install_level(&active_game(&config).await?, &level_dir)
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_remove_level(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  level_name: String,
) -> Result<(), CommandError> {
  active_game(&config)
    .await?
    .remove_level(&level_name)
    .map_err(CommandError::GameFeatures)
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_compile(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  compile(&app_handle, &active_game(&config).await?).await
}

/// The game started at the level's continue point (gk -level).
#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_launch(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  level_name: String,
) -> Result<(), CommandError> {
  launch(active_game(&config).await?, level_name).await
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
