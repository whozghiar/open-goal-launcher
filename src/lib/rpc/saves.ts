import { invoke_rpc } from "./rpc";
import type { SaveInstallInfo } from "./bindings/SaveInstallInfo";
import type { SupportedGame } from "./bindings/SupportedGame";

export async function listGameSaveInstalls(
  gameName: SupportedGame,
): Promise<SaveInstallInfo[]> {
  return await invoke_rpc("list_game_save_installs", { gameName });
}

export async function copySave(
  gameName: SupportedGame,
  fromInstallId: string,
  toInstallId: string,
  fileName: string,
  targetSlot: number | null = null,
  overwrite: boolean = false,
): Promise<void> {
  return await invoke_rpc("copy_save", {
    gameName,
    fromInstallId,
    toInstallId,
    fileName,
    targetSlot,
    overwrite,
  });
}

export async function moveSave(
  gameName: SupportedGame,
  fromInstallId: string,
  toInstallId: string,
  fileName: string,
  targetSlot: number | null = null,
  overwrite: boolean = false,
): Promise<void> {
  return await invoke_rpc("move_save", {
    gameName,
    fromInstallId,
    toInstallId,
    fileName,
    targetSlot,
    overwrite,
  });
}

export async function backupSave(
  gameName: SupportedGame,
  installId: string,
  fileName: string,
): Promise<string> {
  return await invoke_rpc("backup_save", {
    gameName,
    installId,
    fileName,
  });
}

export async function deleteSave(
  gameName: SupportedGame,
  installId: string,
  fileName: string,
): Promise<void> {
  return await invoke_rpc("delete_save", {
    gameName,
    installId,
    fileName,
  });
}

export async function openSaveFolder(
  gameName: SupportedGame,
  installId: string,
): Promise<void> {
  return await invoke_rpc("open_save_folder", {
    gameName,
    installId,
  });
}
