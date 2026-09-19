import { invoke_rpc } from "./rpc";
import type { SaveInstallInfo } from "./bindings/SaveInstallInfo";
import type { SaveFolderInfo } from "./bindings/SaveFolderInfo";
import type { SaveSlotInfo } from "./bindings/SaveSlotInfo";
import type { SupportedGame } from "./bindings/SupportedGame";

export type { SaveInstallInfo, SaveFolderInfo, SaveSlotInfo };

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
  targetFolder: string | null = null,
  targetSlot: number | null = null,
  overwrite: boolean = false,
): Promise<void> {
  return await invoke_rpc("copy_save", {
    gameName,
    fromInstallId,
    toInstallId,
    fileName,
    targetFolder,
    targetSlot,
    overwrite,
  });
}

export async function moveSave(
  gameName: SupportedGame,
  fromInstallId: string,
  toInstallId: string,
  fileName: string,
  targetFolder: string | null = null,
  targetSlot: number | null = null,
  overwrite: boolean = false,
): Promise<void> {
  return await invoke_rpc("move_save", {
    gameName,
    fromInstallId,
    toInstallId,
    fileName,
    targetFolder,
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
  folderName: string | null = null,
): Promise<void> {
  return await invoke_rpc("open_save_folder", {
    gameName,
    installId,
    folderName,
  });
}
