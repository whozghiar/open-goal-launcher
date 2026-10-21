//! What the launcher gives Jak Level Editor (level-editor/, its plugin): the default library, the
//! disc already extracted, ogle-extract downloaded from the editor's release, and the game runtime,
//! the isolated game `<install>/features/jak3/level_edited/` where the edited levels are played.
//! The game's command lines and the level's installation are the editor's (`jle_core::runtime`,
//! `jle_core::io::level_package`): the launcher downloads the runtime, decompiles the disc once and
//! runs the steps as one of its jobs.

use std::{
  future::Future,
  path::{Path, PathBuf},
  process::Stdio,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_jak_level_editor::{
  EDITOR_VERSION, ExtractorProvider, GameRuntime, Host, HostEvent, HostEvents, LevelPackage,
  PlayOutcome, RuntimeInfo,
  jle_core::{
    i18n::tr,
    io::level_package,
    runtime::{self, OpenGoal},
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

// The patched OpenGOAL the edited levels need (OpenGOAL's level builder with the fixes for the
// editor's exports), released on the user's fork. JLE_RUNTIME_URL replaces it (a URL or a local
// archive) for tests.
const RUNTIME_VERSION: &str = "v0.3.9-jle.1";
const RUNTIME_RELEASE: &str = "https://github.com/whozghiar/jak-project/releases/download";

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

// the archives of the fork's release, named as OpenGOAL's own
fn runtime_source() -> Option<String> {
  std::env::var("JLE_RUNTIME_URL").ok().or_else(|| {
    let archive = match platform()? {
      "windows-x64" => "opengoal-windows-{tag}.zip",
      "linux-x64" => "opengoal-linux-{tag}.tar.gz",
      "macos-x64" => "opengoal-macos-intel-{tag}.tar.gz",
      _ => "opengoal-macos-arm-{tag}.tar.gz",
    };
    Some(format!(
      "{RUNTIME_RELEASE}/{RUNTIME_VERSION}/{}",
      archive.replace("{tag}", RUNTIME_VERSION)
    ))
  })
}

fn extractor_dir(install: &Path) -> PathBuf {
  install
    .join("jak-level-editor")
    .join("tools")
    .join(format!("v{EDITOR_VERSION}"))
}

fn game(install: &Path) -> OpenGoal {
  runtime::launcher_level_edited(install, GAME)
}

/// The runtime unpacked in the isolated game, and whether the vanilla disc was decompiled with it.
#[derive(Default, Serialize, Deserialize)]
struct RuntimeState {
  version: String,
  decompiled: bool,
}

impl RuntimeState {
  const FILE: &'static str = "jle-runtime.json";

  fn read(og: &OpenGoal) -> RuntimeState {
    std::fs::read_to_string(og.folder.join(Self::FILE))
      .ok()
      .and_then(|text| serde_json::from_str(&text).ok())
      .unwrap_or_default()
  }

  fn write(&self, og: &OpenGoal) -> anyhow::Result<()> {
    std::fs::write(
      og.folder.join(Self::FILE),
      serde_json::to_string_pretty(self)?,
    )?;
    Ok(())
  }
}

fn runtime_installed(og: &OpenGoal) -> bool {
  RuntimeState::read(og).version == RUNTIME_VERSION && og.gk.is_file()
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

async fn prepare_runtime(
  app: &AppHandle,
  install: &Path,
  mut progress: impl FnMut(u64, u64),
) -> Result<(), CommandError> {
  let og = game(install);
  if runtime_installed(&og) {
    return Ok(());
  }
  let source =
    runtime_source().ok_or_else(|| CommandError::GameFeatures(tr("runtime.none").into()))?;
  log_line(app, format!("Downloading {source}"));
  let log_app = app.clone();
  let mut logged = 0;
  unpack(&source, &og.folder, true, move |done, total| {
    progress(done, total);
    if total > 0 && done * 10 / total > logged {
      logged = done * 10 / total;
      log_line(&log_app, format!("{}%", logged * 10));
    }
  })
  .await?;
  if og.builder(GAME) != runtime::BUILDER {
    return Err(CommandError::GameFeatures(format!(
      "{source}: {}",
      tr("runtime.old_builder")
    )));
  }
  RuntimeState {
    version: RUNTIME_VERSION.into(),
    decompiled: false,
  }
  .write(&og)?;
  Ok(())
}

/// A program of the runtime, its output in the job's logs and the launcher's log folder.
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

async fn decompile(app: &AppHandle, install: &Path) -> Result<(), CommandError> {
  let og = game(install);
  let mut state = RuntimeState::read(&og);
  if state.decompiled {
    return Ok(());
  }
  let mut command = std::process::Command::new(&og.extractor);
  command
    .arg(&og.iso_data)
    .args(["--folder", "--decompile", "--game", GAME])
    .current_dir(&og.folder);
  run(app, command).await?;
  state.decompiled = true;
  state.write(&og)?;
  Ok(())
}

async fn compile(app: &AppHandle, install: &Path) -> Result<(), CommandError> {
  runtime::stop_game();
  run(app, game(install).compile_command(GAME)).await
}

fn install_level(install: &Path, level_dir: &Path) -> Result<(), CommandError> {
  let package = LevelPackage::read(level_dir).map_err(CommandError::GameFeatures)?;
  let og = game(install);
  let builder = &package.manifest.builder;
  if let Some(problem) = og.problem(GAME) {
    return Err(CommandError::GameFeatures(tr(problem).into()));
  }
  if !builder.is_empty() && og.builder(GAME) != *builder {
    return Err(CommandError::GameFeatures(tr("runtime.old_builder").into()));
  }
  level_package::apply(&og.data_dir, &package).map_err(CommandError::GameFeatures)?;
  Ok(())
}

fn installed_levels(install: &Path) -> Vec<String> {
  level_package::installed(&game(install).data_dir)
    .into_iter()
    .filter(|level| level.game == GAME)
    .map(|level| level.name)
    .collect()
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

fn step(events: &HostEvents, key: &str) {
  events(HostEvent::Step {
    key: key.into(),
    args: Vec::new(),
  });
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
  /// info() is asked at each of the editor's get_state: kept a moment
  info: Mutex<Option<(Instant, RuntimeInfo)>>,
}

impl GameRuntime for LevelEdited {
  fn info(&self) -> RuntimeInfo {
    let mut kept = self.info.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, info)) = kept.as_ref()
      && at.elapsed() < Duration::from_secs(2)
    {
      return info.clone();
    }
    let og = game(&self.install);
    let mut info = og.info(GAME);
    info.kind = "launcher".into();
    info.label = "OpenGOAL Launcher".into();
    if !og.iso_data.is_dir() {
      info.available = false;
      info.reason = "runtime.no_game_files".into();
    } else if !runtime_installed(&og) {
      // downloaded and decompiled on the first Play
      info.available = runtime_source().is_some();
      info.reason = if info.available { "" } else { "runtime.none" }.into();
    }
    *kept = Some((Instant::now(), info.clone()));
    info
  }

  fn install_level(&self, package: &LevelPackage, events: HostEvents) -> Result<(), String> {
    let (app, install) = (self.app.clone(), self.install.clone());
    let progress = events.clone();
    let steps = events.clone();
    wait(async move {
      step(&steps, "runtime.step.download");
      prepare_runtime(&app, &install, move |done, total| {
        progress(HostEvent::Progress { done, total })
      })
      .await?;
      step(&steps, "runtime.step.decompile");
      decompile(&app, &install).await
    })?;
    game(&self.install).install_level(package, events)
  }

  fn remove_level(&self, game_name: &str, name: &str, events: HostEvents) -> Result<(), String> {
    game(&self.install).remove_level(game_name, name, events)
  }

  fn launch(&self, game_name: &str, name: &str, events: HostEvents) -> Result<(), String> {
    game(&self.install).launch(game_name, name, events)
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

/// The host the editor gets from the launcher's settings (its installation folder).
pub fn host(app: &AppHandle, install: Option<PathBuf>) -> Host {
  let Some(install) = install else {
    return Host {
      name: "launcher".into(),
      ..Default::default()
    };
  };
  let extractor = extractor_dir(&install).join(exe("ogle-extract"));
  let iso = install
    .join("active")
    .join(GAME)
    .join("data")
    .join("iso_data")
    .join(GAME);
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
      info: Mutex::default(),
    })),
  }
}

async fn install_dir(
  config: &tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<PathBuf, CommandError> {
  Ok(config.lock().await.install_dir()?)
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn refresh_level_editor_host(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  let install = config.lock().await.installation_dir.clone();
  tauri_plugin_jak_level_editor::set_host(&app_handle, host(&app_handle, install));
  Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEditedInfo {
  dir: PathBuf,
  levels: Vec<String>,
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_info(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
) -> Result<LevelEditedInfo, CommandError> {
  let install = install_dir(&config).await?;
  Ok(LevelEditedInfo {
    dir: game(&install).folder,
    levels: installed_levels(&install),
  })
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_prepare_runtime(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  let install = install_dir(&config).await?;
  prepare_runtime(&app_handle, &install, |_, _| {}).await
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_decompile(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  let install = install_dir(&config).await?;
  decompile(&app_handle, &install).await
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_install_level(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  level_dir: PathBuf,
) -> Result<(), CommandError> {
  let install = install_dir(&config).await?;
  install_level(&install, &level_dir)
}

#[instrument(skip(config))]
#[tauri::command]
pub async fn level_edited_remove_level(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  level_name: String,
) -> Result<(), CommandError> {
  let install = install_dir(&config).await?;
  level_package::remove(&game(&install).data_dir, GAME, &level_name)
    .map_err(CommandError::GameFeatures)
}

#[instrument(skip(config, app_handle))]
#[tauri::command]
pub async fn level_edited_compile(
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  app_handle: AppHandle,
) -> Result<(), CommandError> {
  let install = install_dir(&config).await?;
  compile(&app_handle, &install).await
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
  let install = install_dir(&config).await?;
  let events = job_events(&app_handle);
  tauri::async_runtime::spawn_blocking(move || game(&install).launch(GAME, &level_name, events))
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
