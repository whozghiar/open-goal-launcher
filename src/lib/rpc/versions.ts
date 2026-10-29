import { getLatestOfficialRelease } from "$lib/utils/github";
import { invoke_rpc } from "./rpc";
import { config } from "/src/state/config.svelte";

export async function listDownloadedVersions(): Promise<string[]> {
  return await invoke_rpc(
    "list_downloaded_versions",
    { versionFolder: "official" },
    () => [],
  );
}

export async function downloadOfficialVersion(
  version: String,
  url: String,
): Promise<boolean> {
  return await invoke_rpc(
    "download_version",
    { version, url, versionFolder: "official" },
    () => true,
  );
}

export async function removeVersion(version: String): Promise<boolean> {
  return await invoke_rpc("remove_version", { version }, () => true);
}

export async function removeOldVersions(): Promise<boolean> {
  const shouldRemove = config?.deletePreviousVersions;
  if (shouldRemove) {
    let downloadedVersions = await listDownloadedVersions();
    let latestRelease = await getLatestOfficialRelease();
    downloadedVersions = downloadedVersions.filter(
      (v) => v !== latestRelease?.version,
    );
    downloadedVersions.forEach((v) => {
      removeVersion(v);
    });
    return false;
  }
  return false;
}

export async function listDownloadedLevelEditorVersions(): Promise<string[]> {
  return await invoke_rpc(
    "list_downloaded_versions",
    { versionFolder: "level-editor" },
    () => [],
  );
}

export async function downloadLevelEditorVersion(
  version: String,
  url: String,
): Promise<boolean> {
  return await invoke_rpc(
    "download_level_editor_version",
    { version, url },
    () => true,
  );
}

export async function removeLevelEditorVersion(
  version: String,
): Promise<boolean> {
  return await invoke_rpc(
    "remove_level_editor_version",
    { version },
    () => true,
  );
}

export async function listDownloadedAnimationStudioVersions(): Promise<
  string[]
> {
  return await invoke_rpc(
    "list_downloaded_versions",
    { versionFolder: "animation-studio" },
    () => [],
  );
}

export async function downloadAnimationStudioVersion(
  version: String,
  url: String,
): Promise<boolean> {
  return await invoke_rpc(
    "download_animation_studio_version",
    { version, url },
    () => true,
  );
}

export async function removeAnimationStudioVersion(
  version: String,
): Promise<boolean> {
  return await invoke_rpc(
    "remove_animation_studio_version",
    { version },
    () => true,
  );
}

export async function ensureActiveVersionStillExists(): Promise<boolean> {
  return await invoke_rpc("ensure_active_version_still_exists", {});
}
