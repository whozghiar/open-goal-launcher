use std::{
  fs,
  path::{Path, PathBuf},
  time::UNIX_EPOCH,
};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tracing::instrument;
use ts_rs::TS;
use walkdir::WalkDir;

use crate::{
  commands::{game::get_saves_highest_milestone, CommandError},
  config::{LauncherConfig, SupportedGame},
  util::game_milestones::get_jak1_milestones,
};

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/rpc/bindings/")]
pub struct SaveSlotInfo {
  pub file_name: String,
  pub slot_number: Option<u8>,
  pub size_bytes: u64,
  pub modified_timestamp: u64,
  pub milestone_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/rpc/bindings/")]
pub struct SaveInstallInfo {
  pub id: String,
  pub name: String,
  pub is_vanilla: bool,
  pub source_name: Option<String>,
  pub mod_name: Option<String>,
  pub save_dir: String,
  pub has_custom_save_format: bool,
  pub warning_message: Option<String>,
  pub saves: Vec<SaveSlotInfo>,
}

fn check_mod_save_compatibility(mod_name: &str) -> (bool, Option<String>) {
  let lower = mod_name.to_lowercase();
  if lower.contains("fishing") {
    (
      true,
      Some("This mod (Fishing Legacy) alters the save file format. Transferring saves between this mod and vanilla is disabled to protect your save files from corruption.".to_string()),
    )
  } else {
    (false, None)
  }
}

fn parse_slot_number(file_name: &str) -> Option<u8> {
  if file_name.starts_with("jak1-game-") && file_name.ends_with(".bin") {
    let slot_str = &file_name[10..file_name.len() - 4];
    slot_str.parse::<u8>().ok()
  } else if file_name.starts_with("bank") && file_name.ends_with(".bin") {
    let slot_str = &file_name[4..file_name.len() - 4];
    slot_str.parse::<u8>().ok()
  } else {
    None
  }
}

fn scan_saves_in_dir(save_dir: &Path, game_name: SupportedGame) -> Vec<SaveSlotInfo> {
  if !save_dir.exists() {
    return Vec::new();
  }

  let milestones = if game_name == SupportedGame::Jak1 {
    Some(get_jak1_milestones())
  } else {
    None
  };

  let mut saves = Vec::new();
  for entry in WalkDir::new(save_dir)
    .max_depth(2)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_file())
  {
    let path = entry.path();
    let is_bin = path.extension().is_some_and(|ext| ext == "bin");
    if !is_bin {
      continue;
    }

    let file_name = match path.file_name() {
      Some(name) => name.to_string_lossy().into_owned(),
      None => continue,
    };

    let metadata = match fs::metadata(path) {
      Ok(meta) => meta,
      Err(_) => continue,
    };

    let modified_timestamp = metadata
      .modified()
      .ok()
      .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0);

    let milestone_name = if let Some(ref ms) = milestones {
      get_saves_highest_milestone(path, ms).map(|(name, _)| name)
    } else {
      None
    };

    let slot_number = parse_slot_number(&file_name);

    saves.push(SaveSlotInfo {
      file_name,
      slot_number,
      size_bytes: metadata.len(),
      modified_timestamp,
      milestone_name,
    });
  }

  saves.sort_by(|a, b| match (a.slot_number, b.slot_number) {
    (Some(sa), Some(sb)) => sa.cmp(&sb),
    (Some(_), None) => std::cmp::Ordering::Less,
    (None, Some(_)) => std::cmp::Ordering::Greater,
    (None, None) => a.file_name.cmp(&b.file_name),
  });

  saves
}

async fn resolve_install_save_dir(
  app_handle: &tauri::AppHandle,
  config: &tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
  install_id: &str,
) -> Result<PathBuf, CommandError> {
  if install_id == "vanilla" {
    let dir = app_handle
      .path()
      .config_dir()?
      .join("OpenGOAL")
      .join(game_name.to_string())
      .join("saves");
    Ok(dir)
  } else if let Some(stripped) = install_id.strip_prefix("mod:") {
    let parts: Vec<&str> = stripped.split(':').collect();
    if parts.len() != 2 {
      return Err(CommandError::GameManagement(format!("Invalid mod install ID: {install_id}")));
    }
    let source = parts[0];
    let mod_name = parts[1];

    let install_path = {
      let config_lock = config.lock().await;
      config_lock.install_dir()?
    };

    let primary_dir = install_path
      .join("features")
      .join(game_name.to_string())
      .join("mods")
      .join(source)
      .join("_settings")
      .join(mod_name)
      .join("OpenGOAL")
      .join(game_name.to_string())
      .join("saves");

    let fallback_dir = install_path
      .join("features")
      .join(game_name.to_string())
      .join("mods")
      .join(source)
      .join("_settings")
      .join(mod_name)
      .join("saves");

    if primary_dir.exists() || !fallback_dir.exists() {
      Ok(primary_dir)
    } else {
      Ok(fallback_dir)
    }
  } else {
    Err(CommandError::GameManagement(format!("Unknown install ID: {install_id}")))
  }
}

#[instrument(skip(app_handle, config))]
#[tauri::command]
pub async fn list_game_save_installs(
  app_handle: tauri::AppHandle,
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
) -> Result<Vec<SaveInstallInfo>, CommandError> {
  let mut installs = Vec::new();

  let vanilla_save_dir = app_handle
    .path()
    .config_dir()?
    .join("OpenGOAL")
    .join(game_name.to_string())
    .join("saves");

  let vanilla_saves = scan_saves_in_dir(&vanilla_save_dir, game_name);

  installs.push(SaveInstallInfo {
    id: "vanilla".to_string(),
    name: format!("{} (Vanilla)", game_name),
    is_vanilla: true,
    source_name: None,
    mod_name: None,
    save_dir: vanilla_save_dir.to_string_lossy().into_owned(),
    has_custom_save_format: false,
    warning_message: None,
    saves: vanilla_saves,
  });

  let (install_path, installed_mods) = {
    let config_lock = config.lock().await;
    let install_dir = config_lock.install_dir().ok();
    let mods = config_lock
      .games
      .get(&game_name)
      .map(|gc| gc.installed_mods.clone())
      .unwrap_or_default();
    (install_dir, mods)
  };

  if let Some(install_path) = install_path {
    for (source_name, mods_in_source) in &installed_mods {
      for (mod_name, _) in mods_in_source {
        let (has_custom_save_format, warning_message) = check_mod_save_compatibility(mod_name);

        let primary_dir = install_path
          .join("features")
          .join(game_name.to_string())
          .join("mods")
          .join(source_name)
          .join("_settings")
          .join(mod_name)
          .join("OpenGOAL")
          .join(game_name.to_string())
          .join("saves");

        let fallback_dir = install_path
          .join("features")
          .join(game_name.to_string())
          .join("mods")
          .join(source_name)
          .join("_settings")
          .join(mod_name)
          .join("saves");

        let save_dir = if primary_dir.exists() || !fallback_dir.exists() {
          primary_dir
        } else {
          fallback_dir
        };

        let saves = scan_saves_in_dir(&save_dir, game_name);

        installs.push(SaveInstallInfo {
          id: format!("mod:{}:{}", source_name, mod_name),
          name: mod_name.clone(),
          is_vanilla: false,
          source_name: Some(source_name.clone()),
          mod_name: Some(mod_name.clone()),
          save_dir: save_dir.to_string_lossy().into_owned(),
          has_custom_save_format,
          warning_message,
          saves,
        });
      }
    }
  }

  Ok(installs)
}

#[instrument(skip(app_handle, config))]
#[tauri::command]
pub async fn copy_save(
  app_handle: tauri::AppHandle,
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
  from_install_id: String,
  to_install_id: String,
  file_name: String,
  target_slot: Option<u8>,
  overwrite: bool,
) -> Result<(), CommandError> {
  let from_dir = resolve_install_save_dir(&app_handle, &config, game_name, &from_install_id).await?;
  let to_dir = resolve_install_save_dir(&app_handle, &config, game_name, &to_install_id).await?;

  let source_file = from_dir.join(&file_name);
  if !source_file.exists() {
    return Err(CommandError::GameManagement(format!(
      "Source save file does not exist: {}",
      source_file.display()
    )));
  }

  if !to_dir.exists() {
    fs::create_dir_all(&to_dir)?;
  }

  let target_file_name = if let Some(slot) = target_slot {
    if file_name.starts_with("jak1-game-") && file_name.ends_with(".bin") {
      format!("jak1-game-{slot}.bin")
    } else if file_name.starts_with("bank") && file_name.ends_with(".bin") {
      format!("bank{slot}.bin")
    } else {
      file_name.clone()
    }
  } else {
    file_name.clone()
  };

  let target_file = to_dir.join(&target_file_name);

  if target_file.exists() {
    if !overwrite {
      return Err(CommandError::GameManagement("DESTINATION_FILE_EXISTS".to_string()));
    }
    let timestamp = std::time::SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    let bak_path = to_dir.join(format!("{target_file_name}.bak-{timestamp}"));
    let _ = fs::copy(&target_file, bak_path);
  }

  fs::copy(&source_file, &target_file)?;
  tracing::info!("Copied save {} to {}", source_file.display(), target_file.display());
  Ok(())
}

#[instrument(skip(app_handle, config))]
#[tauri::command]
pub async fn move_save(
  app_handle: tauri::AppHandle,
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
  from_install_id: String,
  to_install_id: String,
  file_name: String,
  target_slot: Option<u8>,
  overwrite: bool,
) -> Result<(), CommandError> {
  copy_save(
    app_handle.clone(),
    config.clone(),
    game_name,
    from_install_id.clone(),
    to_install_id,
    file_name.clone(),
    target_slot,
    overwrite,
  )
  .await?;

  let from_dir = resolve_install_save_dir(&app_handle, &config, game_name, &from_install_id).await?;
  let source_file = from_dir.join(&file_name);
  if source_file.exists() {
    fs::remove_file(source_file)?;
  }
  Ok(())
}

#[instrument(skip(app_handle, config))]
#[tauri::command]
pub async fn backup_save(
  app_handle: tauri::AppHandle,
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
  install_id: String,
  file_name: String,
) -> Result<String, CommandError> {
  let save_dir = resolve_install_save_dir(&app_handle, &config, game_name, &install_id).await?;
  let file_path = save_dir.join(&file_name);
  if !file_path.exists() {
    return Err(CommandError::GameManagement(format!(
      "Save file not found: {}",
      file_path.display()
    )));
  }

  let timestamp = std::time::SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();
  let backup_name = format!("{file_name}.bak-{timestamp}");
  let backup_path = save_dir.join(&backup_name);
  fs::copy(&file_path, &backup_path)?;
  tracing::info!("Created backup {}", backup_path.display());
  Ok(backup_name)
}

#[instrument(skip(app_handle, config))]
#[tauri::command]
pub async fn delete_save(
  app_handle: tauri::AppHandle,
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
  install_id: String,
  file_name: String,
) -> Result<(), CommandError> {
  let save_dir = resolve_install_save_dir(&app_handle, &config, game_name, &install_id).await?;
  let file_path = save_dir.join(&file_name);
  if !file_path.exists() {
    return Err(CommandError::GameManagement(format!(
      "Save file not found: {}",
      file_path.display()
    )));
  }

  fs::remove_file(file_path)?;
  tracing::info!("Deleted save file {file_name} from {install_id}");
  Ok(())
}

#[instrument(skip(app_handle, config))]
#[tauri::command]
pub async fn open_save_folder(
  app_handle: tauri::AppHandle,
  config: tauri::State<'_, tokio::sync::Mutex<LauncherConfig>>,
  game_name: SupportedGame,
  install_id: String,
) -> Result<(), CommandError> {
  let save_dir = resolve_install_save_dir(&app_handle, &config, game_name, &install_id).await?;
  if !save_dir.exists() {
    fs::create_dir_all(&save_dir)?;
  }

  #[cfg(target_os = "windows")]
  {
    std::process::Command::new("explorer")
      .arg(&save_dir)
      .spawn()?;
  }
  #[cfg(target_os = "linux")]
  {
    std::process::Command::new("xdg-open")
      .arg(&save_dir)
      .spawn()?;
  }
  #[cfg(target_os = "macos")]
  {
    std::process::Command::new("open")
      .arg(&save_dir)
      .spawn()?;
  }

  Ok(())
}
