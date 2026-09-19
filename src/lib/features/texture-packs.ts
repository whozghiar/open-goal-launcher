export interface PackInfo {
  name: string;
  enabled: boolean;
  toBeDeleted: boolean;
}

export interface PackMetadata {
  fileList: string[];
  coverImagePath: string;
  name: string;
  version: string;
  author: string;
  releaseDate: string;
  description: string;
  tags: string[];
}

export interface AttachedTexturePack {
  name: string;
  downloadUrl: string;
}

import type { ModSourceData } from "$lib/rpc/bindings/ModSourceData";
import type { SupportedGame } from "$lib/rpc/bindings/SupportedGame";
import { platform } from "@tauri-apps/plugin-os";

// Resolves texture packs attached to a specific mod release from the mod source catalog
export function findAttachedTexturePacks(
  sourceData: ModSourceData | undefined | null,
  modName: string,
  modVersion: string | undefined,
  modDownloadUrl: string | undefined,
  activeGame: SupportedGame,
): AttachedTexturePack[] {
  if (
    !sourceData?.texturePacks ||
    Object.keys(sourceData.texturePacks).length === 0
  ) {
    return [];
  }

  let releaseBaseUrl: string | undefined = undefined;
  if (modDownloadUrl) {
    const lastSlash = modDownloadUrl.lastIndexOf("/");
    if (lastSlash !== -1) {
      releaseBaseUrl = modDownloadUrl.substring(0, lastSlash + 1);
    }
  }

  const attachedPacks: AttachedTexturePack[] = [];

  for (const [packKey, packInfo] of Object.entries(sourceData.texturePacks)) {
    // Check if the texture pack supports the active game
    if (
      packInfo.supportedGames &&
      packInfo.supportedGames.length > 0 &&
      !packInfo.supportedGames.includes(activeGame)
    ) {
      continue;
    }

    // Determine if this texture pack is linked to the mod release
    let isAttached = false;

    // Match 1: Asset URL shares the exact same release base URL
    if (releaseBaseUrl && packInfo.versions) {
      isAttached = packInfo.versions.some((v) =>
        Object.values(v.assets || {}).some(
          (url) =>
            url && typeof url === "string" && url.startsWith(releaseBaseUrl!),
        ),
      );
    }

    // Match 2: Texture pack tags include the mod name
    if (!isAttached && packInfo.tags) {
      const targetTag = modName.toLowerCase();
      isAttached = packInfo.tags.some(
        (t) =>
          t.toLowerCase() === targetTag ||
          t.toLowerCase() === `mod:${targetTag}`,
      );
    }

    // Match 3: Texture pack key matches mod name or common prefixes
    if (!isAttached) {
      const lowerKey = packKey.toLowerCase();
      const lowerMod = modName.toLowerCase();
      isAttached =
        lowerKey === lowerMod ||
        lowerKey.startsWith(`${lowerMod}-`) ||
        lowerKey.startsWith(`${lowerMod}_`) ||
        lowerMod.startsWith(`${lowerKey}-`);
    }

    if (!isAttached) {
      continue;
    }

    // Find the best version for the pack
    let selectedVersion = undefined;
    if (releaseBaseUrl && packInfo.versions) {
      selectedVersion = packInfo.versions.find((v) =>
        Object.values(v.assets || {}).some(
          (url) =>
            url && typeof url === "string" && url.startsWith(releaseBaseUrl!),
        ),
      );
    }
    if (!selectedVersion && modVersion && packInfo.versions) {
      selectedVersion = packInfo.versions.find((v) => v.version === modVersion);
    }
    if (!selectedVersion && packInfo.versions && packInfo.versions.length > 0) {
      selectedVersion = packInfo.versions[0];
    }

    if (!selectedVersion?.assets) {
      continue;
    }

    // Resolve the platform-specific download URL
    const plat = platform();
    let downloadUrl: string | undefined = undefined;
    if (selectedVersion.assets[plat]) {
      downloadUrl = selectedVersion.assets[plat]!;
    } else if (selectedVersion.assets["all"]) {
      downloadUrl = selectedVersion.assets["all"]!;
    } else if (selectedVersion.assets["windows"]) {
      downloadUrl = selectedVersion.assets["windows"]!;
    } else {
      const firstValid = Object.values(selectedVersion.assets).find(
        (u) => typeof u === "string" && u.length > 0,
      );
      if (firstValid) {
        downloadUrl = firstValid;
      }
    }

    if (downloadUrl) {
      attachedPacks.push({
        name: packKey,
        downloadUrl,
      });
    }
  }

  return attachedPacks;
}
