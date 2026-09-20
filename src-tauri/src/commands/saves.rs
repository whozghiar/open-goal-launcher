use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  time::UNIX_EPOCH,
};
use tauri::Manager;
use tracing::instrument;
use ts_rs::TS;
use walkdir::WalkDir;

use crate::{
  commands::CommandError,
  config::{LauncherConfig, SupportedGame},
};

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/rpc/bindings/")]
pub struct SaveSlotInfo {
  pub file_name: String,
  pub folder_name: String,
  pub base_name: String,
  pub slot_number: Option<u8>,
  pub size_bytes: u64,
  pub modified_timestamp: u64,
  pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/rpc/bindings/")]
pub struct SaveFolderInfo {
  pub folder_name: String,
  pub display_name: String,
  pub region: Option<String>,
  pub saves: Vec<SaveSlotInfo>,
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
  pub folders: Vec<SaveFolderInfo>,
  pub saves: Vec<SaveSlotInfo>,
}

fn detect_region_and_display(folder_name: &str) -> (Option<String>, String) {
  if folder_name.is_empty() || folder_name == "default" {
    return (None, "default".to_string());
  }

  let upper = folder_name.to_uppercase();
  let region = if upper.contains("BASCUS") || upper.contains("SCUS") {
    Some("NTSC-U".to_string())
  } else if upper.contains("BESCES") || upper.contains("SCES") {
    Some("PAL".to_string())
  } else if upper.contains("SCPS") {
    Some("NTSC-J".to_string())
  } else {
    None
  };

  let display_name = match &region {
    Some(reg) => format!("{folder_name} ({reg})"),
    None => folder_name.to_string(),
  };

  (region, display_name)
}


fn parse_slot_number(file_name: &str) -> Option<u8> {
  let base_name = Path::new(file_name)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or(file_name);

  if base_name.starts_with("jak1-game-") && base_name.ends_with(".bin") {
    let slot_str = &base_name[10..base_name.len() - 4];
    slot_str.parse::<u8>().ok()
  } else if base_name.starts_with("bank") && base_name.ends_with(".bin") {
    let slot_str = &base_name[4..base_name.len() - 4];
    slot_str.parse::<u8>().ok()
  } else {
    None
  }
}

fn scan_save_folders_in_dir(
  save_dir: &Path,
  _game_name: SupportedGame,
) -> (Vec<SaveFolderInfo>, Vec<SaveSlotInfo>) {
  if !save_dir.exists() {
    return (Vec::new(), Vec::new());
  }

  let mut folder_map: BTreeMap<String, Vec<SaveSlotInfo>> = BTreeMap::new();

  if let Ok(entries) = fs::read_dir(save_dir) {
    for entry in entries.filter_map(Result::ok) {
      if let Ok(file_type) = entry.file_type() {
        if file_type.is_dir() {
          let name = entry.file_name().to_string_lossy().into_owned();
          folder_map.entry(name).or_default();
        }
      }
    }
  }

  for entry in WalkDir::new(save_dir)
    .max_depth(3)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_file())
  {
    let path = entry.path();

    let is_bin = path.extension().is_some_and(|ext| ext == "bin");
    if !is_bin {
      continue;
    }

    let rel_path = match path.strip_prefix(save_dir) {
      Ok(rel) => rel,
      Err(_) => continue,
    };

    let file_name = rel_path.to_string_lossy().replace('\\', "/");
    let base_name = path
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_else(|| file_name.clone());

    let folder_name = match rel_path.parent() {
      Some(parent) if parent != Path::new("") => parent.to_string_lossy().replace('\\', "/"),
      _ => "default".to_string(),
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

    let slot_number = parse_slot_number(&base_name);
    let (slot_region, _) = detect_region_and_display(&folder_name);

    let save_slot = SaveSlotInfo {
      file_name,
      folder_name: folder_name.clone(),
      base_name,
      slot_number,
      size_bytes: metadata.len(),
      modified_timestamp,
      region: slot_region,
    };

    folder_map.entry(folder_name).or_default().push(save_slot);
  }

  let mut folders = Vec::new();
  let mut all_saves = Vec::new();

  for (folder_name, mut saves) in folder_map {
    saves.sort_by(|a, b| match (a.slot_number, b.slot_number) {
      (Some(sa), Some(sb)) => sa.cmp(&sb).then_with(|| a.base_name.cmp(&b.base_name)),
      (Some(_), None) => std::cmp::Ordering::Less,
      (None, Some(_)) => std::cmp::Ordering::Greater,
      (None, None) => a.base_name.cmp(&b.base_name),
    });

    let (region, display_name) = detect_region_and_display(&folder_name);

    all_saves.extend(saves.clone());
    folders.push(SaveFolderInfo {
      folder_name,
      display_name,
      region,
      saves,
    });
  }

  folders.sort_by(|a, b| {
    if a.folder_name == "default" {
      std::cmp::Ordering::Less
    } else if b.folder_name == "default" {
      std::cmp::Ordering::Greater
    } else {
      a.display_name.cmp(&b.display_name)
    }
  });

  (folders, all_saves)
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
      return Err(CommandError::GameManagement(format!(
        "Invalid mod install ID: {install_id}"
      )));
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
    Err(CommandError::GameManagement(format!(
      "Unknown install ID: {install_id}"
    )))
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

  let (vanilla_folders, vanilla_saves) = scan_save_folders_in_dir(&vanilla_save_dir, game_name);

  installs.push(SaveInstallInfo {
    id: "vanilla".to_string(),
    name: format!("{} (Vanilla)", game_name),
    is_vanilla: true,
    source_name: None,
    mod_name: None,
    save_dir: vanilla_save_dir.to_string_lossy().into_owned(),
    folders: vanilla_folders,
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

        let (folders, saves) = scan_save_folders_in_dir(&save_dir, game_name);

        installs.push(SaveInstallInfo {
          id: format!("mod:{}:{}", source_name, mod_name),
          name: mod_name.clone(),
          is_vanilla: false,
          source_name: Some(source_name.clone()),
          mod_name: Some(mod_name.clone()),
          save_dir: save_dir.to_string_lossy().into_owned(),
          folders,
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
  target_folder: Option<String>,
  target_slot: Option<u8>,
  overwrite: bool,
) -> Result<(), CommandError> {
  let from_dir =
    resolve_install_save_dir(&app_handle, &config, game_name, &from_install_id).await?;
  let to_dir = resolve_install_save_dir(&app_handle, &config, game_name, &to_install_id).await?;

  let source_file = from_dir.join(&file_name);
  if !source_file.exists() {
    return Err(CommandError::GameManagement(format!(
      "Source save file does not exist: {}",
      source_file.display()
    )));
  }

  let source_parent = Path::new(&file_name).parent();
  let source_folder_name = source_parent
    .and_then(|p| p.to_str())
    .filter(|s| !s.is_empty())
    .unwrap_or("default");
  let (source_region, _) = detect_region_and_display(source_folder_name);

  let target_folder_path = match target_folder.as_deref() {
    Some(f) if !f.is_empty() && f != "default" => PathBuf::from(f),
    _ => PathBuf::new(),
  };

  let target_folder_name = target_folder_path
    .to_str()
    .filter(|s| !s.is_empty())
    .unwrap_or("default");
  let (target_region, _) = detect_region_and_display(target_folder_name);

  if let (Some(s_reg), Some(t_reg)) = (&source_region, &target_region) {
    if s_reg != t_reg {
      return Err(CommandError::GameManagement(format!(
        "Regional incompatibility: Cannot transfer a {s_reg} save to a {t_reg} folder."
      )));
    }
  }

  let base_name = Path::new(&file_name)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or(&file_name);

  let new_base_name = if let Some(slot) = target_slot {
    if base_name.starts_with("jak1-game-") && base_name.ends_with(".bin") {
      format!("jak1-game-{slot}.bin")
    } else if base_name.starts_with("bank") && base_name.ends_with(".bin") {
      format!("bank{slot}.bin")
    } else {
      base_name.to_string()
    }
  } else {
    base_name.to_string()
  };

  let target_rel = if target_folder_path == Path::new("") {
    PathBuf::from(&new_base_name)
  } else {
    target_folder_path.join(&new_base_name)
  };

  let target_file = to_dir.join(&target_rel);

  if let Some(target_parent) = target_file.parent() {
    fs::create_dir_all(target_parent)?;
  }

  if target_file.exists() {
    if !overwrite {
      return Err(CommandError::GameManagement(
        "DESTINATION_FILE_EXISTS".to_string(),
      ));
    }
    let timestamp = std::time::SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    let bak_name = format!("{new_base_name}.bak-{timestamp}");
    let bak_path = target_file.parent().unwrap_or(&to_dir).join(bak_name);
    let _ = fs::copy(&target_file, bak_path);
  }

  fs::copy(&source_file, &target_file)?;
  tracing::info!(
    "Copied save {} to {}",
    source_file.display(),
    target_file.display()
  );
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
  target_folder: Option<String>,
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
    target_folder,
    target_slot,
    overwrite,
  )
  .await?;

  let from_dir =
    resolve_install_save_dir(&app_handle, &config, game_name, &from_install_id).await?;
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
  let base_name = Path::new(&file_name)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or(&file_name);
  let backup_name = format!("{base_name}.bak-{timestamp}");
  let backup_path = file_path.parent().unwrap_or(&save_dir).join(&backup_name);
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
  folder_name: Option<String>,
) -> Result<(), CommandError> {
  let save_dir = resolve_install_save_dir(&app_handle, &config, game_name, &install_id).await?;
  let target_dir = match folder_name.as_deref() {
    Some(sub) if !sub.is_empty() && sub != "default" => save_dir.join(sub),
    _ => save_dir,
  };
  if !target_dir.exists() {
    fs::create_dir_all(&target_dir)?;
  }

  #[cfg(target_os = "windows")]
  {
    std::process::Command::new("explorer")
      .arg(&target_dir)
      .spawn()?;
  }
  #[cfg(target_os = "linux")]
  {
    std::process::Command::new("xdg-open")
      .arg(&target_dir)
      .spawn()?;
  }
  #[cfg(target_os = "macos")]
  {
    std::process::Command::new("open")
      .arg(&target_dir)
      .spawn()?;
  }

  Ok(())
}
