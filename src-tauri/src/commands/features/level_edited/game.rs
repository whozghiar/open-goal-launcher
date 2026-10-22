//! The launcher's isolated game for Jak Level Editor: `<install>/features/jak3/level_edited/`, where
//! the edited levels are installed and played with the programs of the active tooling version
//! (`<install>/versions/official/<version>/`), its own `data/` (copied from the version's, then
//! decompiled from the disc the launcher extracted) and its own `_settings/` (gk's settings and
//! saves): the player's game and mods are never touched. Each level installed is also kept in
//! `packages/<name>/`, to install it again in the data of another version.

use std::{
  path::{Path, PathBuf},
  process::{Child, Command, Stdio},
  sync::Mutex,
  time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tauri_plugin_jak_level_editor::{LevelPackage, jle_core::io::level_package};

pub const GAME: &str = "jak3";

/// The last tooling version whose gk cannot start the game at a continue point (`-level`): the
/// game needs a release of whozghiar/jak-project newer than it.
const NO_LEVEL_OPTION: &str = "0.3.9-whozghiar.1";

pub fn exe(dir: &Path, name: &str) -> PathBuf {
  dir.join(if cfg!(windows) {
    format!("{name}.exe")
  } else {
    name.to_owned()
  })
}

/// The game's files the launcher extracted (read only: building from them writes nothing there)
pub fn launcher_iso_data(install: &Path) -> PathBuf {
  install
    .join("active")
    .join(GAME)
    .join("data")
    .join("iso_data")
    .join(GAME)
}

/// A release of whozghiar/jak-project whose gk takes `-level`
fn starts_at_a_level(version: &str) -> bool {
  let v = |s: &str| semver::Version::parse(s.strip_prefix('v').unwrap_or(s)).ok();
  version.contains("-whozghiar.") && v(version) > v(NO_LEVEL_OPTION)
}

/// What the game was made from (`jle-runtime.json` in its folder): the tooling version its data/
/// was copied from, and whether the disc was decompiled into it since.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeState {
  pub version: String,
  pub decompiled: bool,
}

impl RuntimeState {
  const FILE: &'static str = "jle-runtime.json";

  pub fn read(folder: &Path) -> RuntimeState {
    std::fs::read(folder.join(Self::FILE))
      .ok()
      .and_then(|b| serde_json::from_slice(&b).ok())
      .unwrap_or_default()
  }

  pub fn write(&self, folder: &Path) -> Result<(), String> {
    let path = folder.join(Self::FILE);
    let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
  }
}

fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
  if let Some(dir) = to.parent() {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
  }
  std::fs::copy(from, to)
    .map(|_| ())
    .map_err(|e| format!("{}: {e}", from.display()))
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
  let entries = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
  std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
  for entry in entries {
    let entry = entry.map_err(|e| e.to_string())?;
    let (src, dst) = (entry.path(), to.join(entry.file_name()));
    if src.is_dir() {
      copy_dir(&src, &dst)?;
    } else {
      copy_file(&src, &dst)?;
    }
  }
  Ok(())
}

/// the game started last: stopped before another one starts, or a build
static GAME_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

/// Stops the game started last, when it still runs (it keeps the files of its data open).
pub fn stop_game() {
  if let Some(mut child) = GAME_PROCESS
    .lock()
    .unwrap_or_else(|e| e.into_inner())
    .take()
    && child.try_wait().ok().flatten().is_none()
  {
    let _ = child.kill();
    let _ = child.wait();
  }
}

/// The isolated game with the programs of a tooling version.
#[derive(Clone, Debug)]
pub struct EditedGame {
  pub folder: PathBuf,
  pub data_dir: PathBuf,
  /// gk's settings and saves, apart from the player's game
  pub config_dir: PathBuf,
  /// the disc's files it builds from
  pub iso_data: PathBuf,
  pub version: String,
  /// the version's programs and data/
  pub tooling_dir: PathBuf,
}

impl EditedGame {
  pub fn new(install: &Path, version: &str) -> EditedGame {
    let folder = install.join("features").join(GAME).join("level_edited");
    EditedGame {
      data_dir: folder.join("data"),
      config_dir: folder.join("_settings"),
      iso_data: launcher_iso_data(install),
      version: version.to_owned(),
      tooling_dir: install.join("versions").join("official").join(version),
      folder,
    }
  }

  fn program(&self, name: &str) -> PathBuf {
    exe(&self.tooling_dir, name)
  }

  /// Why it cannot play a level now (a translation key of the editor), None when it can
  pub fn problem(&self) -> Option<&'static str> {
    if !starts_at_a_level(&self.version)
      || !["gk", "extractor"]
        .iter()
        .all(|p| self.program(p).is_file())
    {
      return Some("runtime.launcher_version");
    }
    if !self.iso_data.join("CGO").is_dir() {
      return Some("runtime.no_game_files");
    }
    None
  }

  /// where it keeps a copy of each level installed
  fn packages_dir(&self) -> PathBuf {
    self.folder.join("packages")
  }

  /// It must be made again from its version: its data/ was made from another one, or not yet
  pub fn stale(&self) -> bool {
    RuntimeState::read(&self.folder).version != self.version
      || !self.data_dir.join("goal_src").is_dir()
  }

  /// The levels installed (the ones it installs again when it is stale)
  pub fn installed(&self) -> Vec<String> {
    if !self.stale() {
      return level_package::installed(&self.data_dir, GAME);
    }
    let mut names: Vec<String> = std::fs::read_dir(self.packages_dir())
      .into_iter()
      .flatten()
      .flatten()
      .filter(|e| e.path().join(LevelPackage::MANIFEST).is_file())
      .map(|e| e.file_name().to_string_lossy().into_owned())
      .collect();
    names.sort();
    names
  }

  /// Made from its version when it is stale: data/ copied again from the version's (_settings/,
  /// the saves, kept), to be decompiled again, and the levels installed before installed again
  /// from their copies. Returns what it is made from.
  pub fn prepare(&self) -> Result<RuntimeState, String> {
    if !self.stale() {
      return Ok(RuntimeState::read(&self.folder));
    }
    stop_game();
    if self.data_dir.exists() {
      std::fs::remove_dir_all(&self.data_dir)
        .map_err(|e| format!("{}: {e}", self.data_dir.display()))?;
    }
    copy_dir(&self.tooling_dir.join("data"), &self.data_dir)?;
    for name in self.installed() {
      level_package::apply(
        &self.data_dir,
        &LevelPackage::read(&self.packages_dir().join(name))?,
      )?;
    }
    // written last: a preparation stopped halfway starts again
    let state = RuntimeState {
      version: self.version.clone(),
      decompiled: false,
    };
    state.write(&self.folder)?;
    Ok(state)
  }

  /// The disc decompiled into its data (`extractor --folder --decompile`), once per version
  pub fn decompile_command(&self) -> Command {
    self.extractor("--decompile")
  }

  /// The game built again, its levels included (`extractor --folder --compile`: goalc's `(mi)`)
  pub fn compile_command(&self) -> Command {
    self.extractor("--compile")
  }

  fn extractor(&self, action: &str) -> Command {
    let mut cmd = Command::new(self.program("extractor"));
    cmd
      .arg(&self.iso_data)
      .args(["--folder", action, "--game", GAME, "--proj-path"])
      .arg(&self.data_dir)
      .current_dir(&self.tooling_dir);
    cmd
  }

  /// Installs the level in its data (level_package::apply) and keeps a copy of it
  pub fn install_level(&self, package: &LevelPackage) -> Result<(), String> {
    level_package::apply(&self.data_dir, package)?;
    let copy = self.packages_dir().join(&package.manifest.name);
    if copy == package.dir {
      return Ok(());
    }
    let _ = std::fs::remove_dir_all(&copy);
    let m = &package.manifest;
    let files = std::iter::once(LevelPackage::MANIFEST)
      .chain(m.files.iter().map(String::as_str))
      .chain((!m.level_info.is_empty()).then_some(m.level_info.as_str()));
    for f in files {
      if !Path::new(f)
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
      {
        return Err(format!("jle-level.json: a file outside the export ({f})"));
      }
      copy_file(&package.dir.join(f), &copy.join(f))?;
    }
    Ok(())
  }

  /// Takes the level out of its data (level_package::remove), and its copy
  pub fn remove_level(&self, name: &str) -> Result<(), String> {
    let copy = self.packages_dir().join(name);
    let had_copy = copy.is_dir();
    let _ = std::fs::remove_dir_all(&copy);
    if had_copy
      && !level_package::installed(&self.data_dir, GAME)
        .iter()
        .any(|n| n == name)
    {
      return Ok(());
    }
    level_package::remove(&self.data_dir, GAME, name)
  }

  /// gk started at the continue point of an installed level (`-level`), its settings and saves its
  /// own, its output in `_settings/jle-game.log`; an error when it stops at once.
  pub fn launch(&self, name: &str) -> Result<(), String> {
    let package = LevelPackage::read(&self.packages_dir().join(name))?;
    stop_game();
    std::fs::create_dir_all(&self.config_dir)
      .map_err(|e| format!("{}: {e}", self.config_dir.display()))?;
    let log = self.config_dir.join("jle-game.log");
    let out = std::fs::File::create(&log).map_err(|e| format!("{}: {e}", log.display()))?;
    let mut cmd = Command::new(self.program("gk"));
    cmd
      .args(["-v", "--game", GAME, "--proj-path"])
      .arg(&self.data_dir)
      .arg("--config-path")
      .arg(&self.config_dir)
      // a debug game (its larger level heaps), started at the level
      .args(["--", "-boot", "-fakeiso", "-debug", "-level"])
      .arg(&package.manifest.continue_point)
      .current_dir(&self.tooling_dir)
      .stdin(Stdio::null())
      .stdout(out.try_clone().map_err(|e| e.to_string())?)
      .stderr(out);
    #[cfg(windows)]
    {
      use std::os::windows::process::CommandExt;
      cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console window of its own
    }
    tracing::info!("Running {cmd:?}");
    let mut gk = cmd
      .spawn()
      .map_err(|e| format!("{}: {e}", self.program("gk").display()))?;
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
      if let Ok(Some(status)) = gk.try_wait() {
        let text = std::fs::read_to_string(&log).unwrap_or_default();
        let tail: Vec<&str> = text.lines().rev().take(15).collect();
        let tail: Vec<&str> = tail.into_iter().rev().collect();
        return Err(format!("gk: {status}\n{}", tail.join("\n")));
      }
      std::thread::sleep(Duration::from_millis(100));
    }
    *GAME_PROCESS.lock().unwrap_or_else(|e| e.into_inner()) = Some(gk);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tauri_plugin_jak_level_editor::LevelManifest;

  fn exported(dir: &Path, name: &str) -> LevelPackage {
    let level = format!("custom_assets/jak3/levels/{name}");
    std::fs::create_dir_all(dir.join(&level)).unwrap();
    std::fs::write(dir.join(format!("{level}/{name}.glb")), "glb").unwrap();
    std::fs::write(
      dir.join(format!("{level}/level-info.gc")),
      format!("(define {name}\n  (new 'static 'level-load-info\n    :name '{name}\n    :index #x128\n    ))\n"),
    )
    .unwrap();
    let manifest = LevelManifest {
      format: 1,
      game: GAME.into(),
      name: name.into(),
      nickname: "mte".into(),
      continue_point: format!("{name}-start"),
      game_gp: vec![format!("(build-custom-level \"{name}\")")],
      level_info: format!("{level}/level-info.gc"),
      files: vec![format!("{level}/{name}.glb")],
      ..Default::default()
    };
    std::fs::write(
      dir.join(LevelPackage::MANIFEST),
      serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
    LevelPackage::read(dir).unwrap()
  }

  /// a version's folder: its programs and a small data/
  fn version(install: &Path, v: &str) {
    let dir = install.join("versions/official").join(v);
    for p in ["gk", "extractor"] {
      std::fs::create_dir_all(&dir).unwrap();
      std::fs::write(exe(&dir, p), "").unwrap();
    }
    let gp = dir.join("data/goal_src/jak3/game.gp");
    std::fs::create_dir_all(dir.join("data/goal_src/jak3/engine/level")).unwrap();
    std::fs::write(
      &gp,
      "(custom-level-cgo \"TSZ.DGO\" \"test-zone/testzone.gd\")\n(group-list \"iso\")\n",
    )
    .unwrap();
    std::fs::write(
      dir.join("data/goal_src/jak3/engine/level/level-info.gc"),
      format!("(define test-zone\n  :index #x128\n  ) ;; {v}\n"),
    )
    .unwrap();
  }

  #[test]
  fn the_isolated_game_follows_the_active_version() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path();
    let (v1, v2) = ("v0.3.9-whozghiar.2", "v0.3.10-whozghiar.1");
    version(install, v1);
    version(install, v2);
    std::fs::create_dir_all(launcher_iso_data(install).join("CGO")).unwrap();

    assert_eq!(
      EditedGame::new(install, "v0.3.9-whozghiar.1").problem(),
      Some("runtime.launcher_version"),
      "its gk has no -level"
    );
    assert_eq!(
      EditedGame::new(install, "v0.3.9").problem(),
      Some("runtime.launcher_version"),
      "not a release of the fork"
    );
    let game = EditedGame::new(install, v1);
    assert!(game.problem().is_none() && game.stale());
    assert!(!game.prepare().unwrap().decompiled && !game.stale());
    game
      .install_level(&exported(&tmp.path().join("export"), "my-test"))
      .unwrap();
    assert_eq!(game.installed(), ["my-test"]);
    assert!(
      game
        .folder
        .join("packages/my-test/custom_assets/jak3/levels/my-test/my-test.glb")
        .is_file()
    );

    // another version: its data again, the level installed again from its copy
    let next = EditedGame::new(install, v2);
    assert!(next.stale() && next.installed() == ["my-test"]);
    next.prepare().unwrap();
    let info = std::fs::read_to_string(
      next
        .data_dir
        .join("goal_src/jak3/engine/level/level-info.gc"),
    )
    .unwrap();
    assert!(
      info.contains(v2) && info.contains("(define my-test"),
      "{info}"
    );
    assert_eq!(level_package::installed(&next.data_dir, GAME), ["my-test"]);

    next.remove_level("my-test").unwrap();
    assert!(next.installed().is_empty() && !next.folder.join("packages/my-test").exists());
  }
}
