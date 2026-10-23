import { toastStore } from "$lib/stores/ToastStore";
import { arch, platform } from "@tauri-apps/plugin-os";
import { unwrapFunctionStore, format } from "svelte-i18n";
import type { RestEndpointMethodTypes } from "@octokit/rest";

type GitHubRelease =
  RestEndpointMethodTypes["repos"]["getRelease"]["response"]["data"];

const $format = unwrapFunctionStore(format);

// the OpenGOAL tooling the launcher installs and updates: this fork of jak-project carries the
// level builder's fixes the Level Editor's levels need, with releases named as upstream's
export const TOOLING_REPOSITORY = "whozghiar/jak-project";

// Jak Level Editor, installed and updated apart from the launcher
export const LEVEL_EDITOR_REPOSITORY = "whozghiar/jak-level-editor";

type DownloadLink = (githubRelease: GitHubRelease) => string | undefined;

export interface ReleaseInfo {
  version: string;
  date: string | undefined;
  githubLink: string | undefined;
  downloadUrl: string | undefined;
  isDownloaded: boolean;
  pendingAction: boolean;
  invalid: boolean;
  invalidationReasons: string[];
}

function getDownloadLinkForCurrentPlatform(githubRelease: GitHubRelease) {
  let plat = platform();
  let matchingAsset;
  if (plat == "macos") {
    const userArch = arch() === "aarch64" ? "arm" : "intel";
    matchingAsset = githubRelease.assets.find(
      (asset) =>
        asset.name.toLowerCase().includes(plat) &&
        !asset.name.toLowerCase().includes(".bin") &&
        !asset.name.toLowerCase().includes("lsp") &&
        asset.name.toLowerCase().includes(userArch),
    );
  } else {
    matchingAsset = githubRelease.assets.find(
      (asset) =>
        asset.name.toLowerCase().includes(plat) &&
        !asset.name.toLowerCase().includes(".bin") &&
        !asset.name.toLowerCase().includes("lsp"),
    );
  }
  if (matchingAsset) {
    return matchingAsset.browser_download_url;
  }
  return undefined;
}

// the editor's portable build, its ogle-extract included; none for macOS
function getLevelEditorDownloadLink(githubRelease: GitHubRelease) {
  const plat = platform() === "windows" ? "windows-x64" : "linux-x64";
  return githubRelease.assets.find(
    (asset) => asset.name === `jak-level-editor-portable-${plat}.zip`,
  )?.browser_download_url;
}

async function parseGithubRelease(
  githubRelease: GitHubRelease,
  downloadLink: DownloadLink,
): Promise<ReleaseInfo> {
  const releaseInfo: ReleaseInfo = {
    version: githubRelease.tag_name,
    date:
      githubRelease.published_at === null
        ? undefined
        : githubRelease.published_at,
    githubLink: githubRelease.html_url,
    downloadUrl: downloadLink(githubRelease),
    isDownloaded: false,
    pendingAction: false,
    invalid: false,
    invalidationReasons: [],
  };
  if (githubRelease.body && githubRelease.body.includes("<!-- invalid:")) {
    releaseInfo.invalid = true;
    // Get the line it's on
    try {
      const line = githubRelease.body
        .split("<!-- invalid:")[1]
        .split("-->")[0]
        .trim();
      releaseInfo.invalidationReasons = line.split("|");
    } catch (err) {
      // do nothing, bad formatting
      releaseInfo.invalidationReasons = ["Release invalid for unknown reasons"];
    }
  }

  return releaseInfo;
}

export async function listOfficialReleases(): Promise<ReleaseInfo[]> {
  return listReleases(TOOLING_REPOSITORY, getDownloadLinkForCurrentPlatform);
}

export async function listLevelEditorReleases(): Promise<ReleaseInfo[]> {
  return listReleases(LEVEL_EDITOR_REPOSITORY, getLevelEditorDownloadLink);
}

export async function getLatestOfficialRelease(): Promise<
  ReleaseInfo | undefined
> {
  return getLatestRelease(
    TOOLING_REPOSITORY,
    getDownloadLinkForCurrentPlatform,
  );
}

export async function getLatestLevelEditorRelease(): Promise<
  ReleaseInfo | undefined
> {
  return getLatestRelease(LEVEL_EDITOR_REPOSITORY, getLevelEditorDownloadLink);
}

async function listReleases(
  repository: string,
  downloadLink: DownloadLink,
): Promise<ReleaseInfo[]> {
  const nextUrlPattern = /<([\S]+)>; rel="Next"/i;
  let releases = [];
  let urlToHit: string | undefined =
    `https://api.github.com/repos/${repository}/releases?per_page=100`;

  while (urlToHit !== undefined) {
    const resp: Response = await fetch(urlToHit);
    if (resp.status === 429) {
      toastStore.makeToast($format("toasts_githubRateLimit"), "error");
      return [];
    } else if (resp.status === 404) {
      return [];
    } else if (!resp.ok) {
      toastStore.makeToast(resp.statusText, "error");
      return [];
    }

    const githubReleases: GitHubRelease[] = await resp.json();
    for (const release of githubReleases) {
      releases.push(await parseGithubRelease(release, downloadLink));
    }

    const linkHeader = resp.headers.get("link");
    if (linkHeader && linkHeader.includes(`rel=\"next\"`)) {
      const match = linkHeader.match(nextUrlPattern);
      if (match && match.length >= 2) {
        // we must paginate!
        urlToHit = match[1];
      } else {
        urlToHit = undefined;
      }
    } else {
      urlToHit = undefined;
    }
  }
  return releases.sort((a, b) => {
    const aDate = a.date;
    const bDate = b.date;
    if (aDate && bDate) {
      return bDate.localeCompare(aDate);
    }
    return 0;
  });
}

async function getLatestRelease(
  repository: string,
  downloadLink: DownloadLink,
): Promise<ReleaseInfo | undefined> {
  const resp = await fetch(
    `https://api.github.com/repos/${repository}/releases/latest`,
  );
  if (resp.status === 429) {
    toastStore.makeToast($format("toasts_githubRateLimit"), "error");
    return undefined;
  } else if (resp.status === 404) {
    // a repository without any release yet: no update
    return undefined;
  } else if (!resp.ok) {
    toastStore.makeToast(resp.statusText, "error");
    return undefined;
  }
  const githubRelease = await resp.json();
  return await parseGithubRelease(githubRelease, downloadLink);
}
