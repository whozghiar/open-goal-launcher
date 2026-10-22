//! What the launcher gives Jak Level Editor (level-editor/, its plugin): the default library, the
//! disc already extracted, ogle-extract downloaded from the editor's release, and the game runtime,
//! the isolated game `<install>/features/jak3/level_edited/` where the edited levels are played
//! with the programs of the active tooling version (a release of whozghiar/jak-project, whose level
//! builder has the fixes the levels need). The game's preparation for a version, its command lines
//! and the level's installation are the editor's (`jle_core::runtime`, `jle_core::io::level_package`):
//! the launcher runs the steps as one of its jobs.

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
  PlayOutcome, RuntimeInfo,
  jle_core::{
    i18n::tr,
    runtime::{self, OpenGoal, RuntimeState},
  },
};
use tracing::instrument;

use crate::{
  commands::{CommandError, binaries::format_exit_code},
  config::LauncherConfig,
  util::{
    file::delete_dir,
    network::download_file_with_progress,
    process::{create_log_file, watch_process},
    tar::extract_and_delete_archive,
  },
};

const GAME: &str = "jak3";

const EXTRACTOR_URL: &str = "https://github.com/whozghiar/open-goal-level-builder/releases/download/v{version}/ogle-extract-{platform}.zip";

fn exe(name: &str) -> String {
  if cfg!(windows) {
    format!("{name}.exe")
  } else {
    name.to_string()
  }
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

/// The isolated game run with the programs of the tooling version `version`
fn game(install: &Path, version: &str) -> OpenGoal {
  runtime::launcher_level_edited(install, version, GAME)
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

/// A release's archive (a URL, or a local file for tests) unpacked into `dest`.
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
  if source.starts_with("http://") || source.starts_with("https://") {
    download_file_with_progress(source, &archive, throttled(progress)).await?;
  } else {
    std::fs::copy(source, &archive).with_context(|| format!("Unable to read {source}"))?;
  }
  extract_and_delete_archive(&archive, dest, strip_top_dir)
}

fn log_line(app: &AppHandle, line: String) {
  let _ = app.emit("log_update", serde_json::json!({ "logs": line }));
}

/// The editor's runtime reporting to the job's logs.
fn job_events(app: &AppHandle) -> HostEvents {
  let app = app.clone();
  Arc::new(move |event| match event {
    HostEvent::Log { line } => log_line(&app, line),
    HostEvent::Step { key, args } => {
      let mut text = tr(&key).to_string();
      for arg in args {
        text = text.replacen("%s", &arg, 1);
      }
      log_line(&app, text);
    }
    HostEvent::Progress { .. } => {}
  })
}

/// The isolated game made from its tooling version when it was made from another one (its data
/// copied again, its levels installed again); it then still has to be decompiled.
async fn prepare(app: &AppHandle, og: OpenGoal) -> Result<RuntimeState, CommandError> {
  if let Some(problem) = og.problem(GAME) {
    return Err(CommandError::GameFeatures(tr(problem).into()));
  }
  if og.stale()
    && let Some(tooling) = &og.tooling
  {
    log_line(
      app,
      tr("runtime.step.prepare").replacen("%s", &tooling.version, 1),
    );
  }
  tauri::async_runtime::spawn_blocking(move || og.prepare())
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

async fn decompile(app: &AppHandle, og: OpenGoal) -> Result<(), CommandError> {
  let mut state = prepare(app, og.clone()).await?;
  if state.decompiled {
    return Ok(());
  }
  run(app, og.decompile_command(GAME)).await?;
  state.decompiled = true;
  state.write(&og.folder).map_err(CommandError::GameFeatures)
}

async fn compile(app: &AppHandle, og: &OpenGoal) -> Result<(), CommandError> {
  runtime::stop_game();
  run(app, og.compile_command(GAME)).await
}

fn install_level(og: &OpenGoal, level_dir: &Path) -> Result<(), CommandError> {
  let package = LevelPackage::read(level_dir).map_err(CommandError::GameFeatures)?;
  if let Some(problem) = og.problem(GAME) {
    return Err(CommandError::GameFeatures(tr(problem).into()));
  }
  og.apply_level(&package).map_err(CommandError::GameFeatures)
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
    let path = dir.join(exe("ogle-extract"));
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
      Err(format!(
        "No {} in the editor's release",
        exe("ogle-extract")
      ))
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
  fn game(&self) -> OpenGoal {
    game(&self.install, &self.version)
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
    let mut info = self.game().info(GAME);
    info.kind = "launcher".into();
    info.label = "OpenGOAL Launcher".into();
    *kept = Some((Instant::now(), info.clone()));
    info
  }

  fn install_level(&self, package: &LevelPackage, events: HostEvents) -> Result<(), String> {
    self.game().install_level(package, events)
  }

  fn remove_level(&self, game_name: &str, name: &str, events: HostEvents) -> Result<(), String> {
    self.game().remove_level(game_name, name, events)
  }

  fn launch(&self, game_name: &str, name: &str, events: HostEvents) -> Result<(), String> {
    self.game().launch(game_name, name, events)
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
  let extractor = extractor_dir(&install).join(exe("ogle-extract"));
  let iso = runtime::launcher_iso_data(&install, GAME);
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
) -> Result<OpenGoal, CommandError> {
  let config = config.lock().await;
  Ok(game(
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
  let og = active_game(&config).await?;
  let info = og.info(GAME);
  Ok(LevelEditedInfo {
    levels: info.installed,
    version: RuntimeState::read(&og.folder).version,
    active_version: og
      .tooling
      .as_ref()
      .map(|t| t.version.clone())
      .unwrap_or_default(),
    stale: og.stale(),
    problem: og.problem(GAME).map(|p| tr(p).to_string()),
    dir: og.folder,
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
    .remove_level_files(GAME, &level_name)
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

/// The game started at the level's continue point (the editor's runtime: gk, then goalc taking Jak
/// there).
#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_launch(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
  level_name: String,
) -> Result<(), CommandError> {
  let og = active_game(&config).await?;
  let events = job_events(&app_handle);
  tauri::async_runtime::spawn_blocking(move || og.launch(GAME, &level_name, events))
    .await
    .map_err(|e| CommandError::GameFeatures(e.to_string()))?
    .map_err(CommandError::GameFeatures)
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
    zip.start_file(exe("ogle-extract"), options).unwrap();
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
    assert_eq!(path, dir.join(exe("ogle-extract")));
    assert!(dir.join("ogle-extract-data/game.json").is_file());
    assert!(!dir.join("ogle-extract-test.zip").exists());
    assert_eq!(provider.fetch(Arc::new(|_| {})).unwrap(), path);
  }
}
