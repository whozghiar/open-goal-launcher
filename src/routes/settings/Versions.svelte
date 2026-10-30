<script lang="ts">
  import { onMount } from "svelte";
  import {
    downloadAnimationStudioVersion,
    downloadLevelEditorVersion,
    downloadOfficialVersion,
    listDownloadedAnimationStudioVersions,
    listDownloadedLevelEditorVersions,
    listDownloadedVersions,
    removeAnimationStudioVersion,
    removeLevelEditorVersion,
    removeVersion,
  } from "$lib/rpc/versions";
  import {
    listAnimationStudioReleases,
    listLevelEditorReleases,
    listOfficialReleases,
    type ReleaseInfo,
  } from "$lib/utils/github";
  import VersionList from "./versions/VersionList.svelte";
  import { UpdateStore } from "$lib/stores/AppStore";
  import {
    setActiveVersion,
    setAnimationStudioVersion,
    setLevelEditorVersion,
  } from "$lib/rpc/config";
  import { _ } from "svelte-i18n";
  import { toastStore } from "$lib/stores/ToastStore";
  import { versionState } from "/src/state/VersionState.svelte";
  import { Alert, Button, Spinner } from "flowbite-svelte";
  import Latest from "./versions/Latest.svelte";
  import { openPath } from "@tauri-apps/plugin-opener";
  import IconFolderOpen from "~icons/mdi/folder-open";
  import IconRefresh from "~icons/mdi/refresh";
  import { config } from "/src/state/config.svelte";

  // the tooling, Jak Level Editor or Jak Animation Studio: each downloaded per version from
  // its GitHub releases
  let {
    kind = "tooling",
  }: { kind?: "tooling" | "levelEditor" | "animationStudio" } = $props();

  const sources = {
    tooling: {
      activeKey: "activeToolingVersion",
      updateKey: "selectedTooling",
      configured: () => config?.activeVersion,
      folder: "/versions/official/",
      header: "settings_versions_header",
      description: "settings_versions_official_description",
      savedToast: "toasts_savedToolingVersion",
      listDownloaded: listDownloadedVersions,
      listReleases: listOfficialReleases,
      download: downloadOfficialVersion,
      remove: removeVersion,
      setActive: setActiveVersion,
    },
    levelEditor: {
      activeKey: "activeLevelEditorVersion",
      updateKey: "selectedLevelEditor",
      configured: () => config?.levelEditorVersion,
      folder: "/versions/level-editor/",
      header: "settings_versions_levelEditor_header",
      description: "settings_versions_levelEditor_description",
      savedToast: "toasts_savedLevelEditorVersion",
      listDownloaded: listDownloadedLevelEditorVersions,
      listReleases: listLevelEditorReleases,
      download: downloadLevelEditorVersion,
      remove: removeLevelEditorVersion,
      setActive: setLevelEditorVersion,
    },
    animationStudio: {
      activeKey: "activeAnimationStudioVersion",
      updateKey: "selectedAnimationStudio",
      configured: () => config?.animationStudioVersion,
      folder: "/versions/animation-studio/",
      header: "settings_versions_animationStudio_header",
      description: "settings_versions_animationStudio_description",
      savedToast: "toasts_savedAnimationStudioVersion",
      listDownloaded: listDownloadedAnimationStudioVersions,
      listReleases: listAnimationStudioReleases,
      download: downloadAnimationStudioVersion,
      remove: removeAnimationStudioVersion,
      setActive: setAnimationStudioVersion,
    },
  } as const;
  const source = $derived(sources[kind]);

  let loading = $state(true);
  let releases: ReleaseInfo[] = $state([]);

  onMount(async () => {
    await refreshVersionList();
    loading = false;
  });

  async function refreshVersionList() {
    loading = true;
    versionState[source.activeKey] = source.configured();
    // Check the backend to see if the folder has any versions
    const installedVersions = await source.listDownloaded();
    releases = [];
    for (const version of installedVersions) {
      releases = [
        ...releases,
        {
          version: version,
          date: undefined,
          githubLink: undefined,
          downloadUrl: undefined,
          isDownloaded: true,
          pendingAction: false,
          invalid: false,
          invalidationReasons: [],
        },
      ];
    }

    // Merge that with the actual current releases on github
    const githubReleases = await source.listReleases();
    for (const release of githubReleases) {
      // Look to see if we already have this release downloaded and we just have to fill in some metadata about it
      let foundExistingRelease = false;
      for (const existingRelease of releases) {
        if (existingRelease.version === release.version) {
          existingRelease.date = release.date;
          existingRelease.githubLink = release.githubLink;
          existingRelease.downloadUrl = release.downloadUrl;
          foundExistingRelease = true;
          break;
        }
      }
      if (foundExistingRelease) {
        continue;
      }
      releases = [
        ...releases,
        {
          version: release.version,
          date: release.date,
          githubLink: release.githubLink,
          downloadUrl: release.downloadUrl,
          isDownloaded: false,
          pendingAction: false,
          invalid: release.invalid,
          invalidationReasons: release.invalidationReasons,
        },
      ];
    }

    // filter incompatible releases; installed versions stay even when GitHub does not list them
    // (a build of the whozghiar/jak-project fork, installed by launcher 1.1)
    releases = releases.filter(
      (r) => r.isDownloaded || r.downloadUrl !== undefined,
    );
    releases = releases.filter((r) => !r.invalid);

    // if no releases are found, early out
    if (releases.length > 0) {
      // Sort releases by published date
      releases = releases.sort((a, b) => {
        if (a.date === undefined) {
          return 1;
        }
        if (b.date === undefined) {
          return -1;
        }
        return b.date.localeCompare(a.date);
      });

      // If we find the latest when refreshing, get rid of the notification
      if ($UpdateStore[source.updateKey].updateAvailable) {
        $UpdateStore[source.updateKey].updateAvailable =
          !releases[0].isDownloaded;
      }
    }

    loading = false;
  }

  async function saveOfficialVersionChange(version: string) {
    const error = await source.setActive(version);
    if (!error) {
      toastStore.makeToast($_(source.savedToast), "info");
    }
  }

  async function onDownloadVersion(version: string, downloadUrl: string) {
    // Mark that release as being downloaded
    releases = releases.map((release) =>
      release.version === version
        ? { ...release, pendingAction: true }
        : release,
    );
    const error = await source.download(version, downloadUrl);
    if (error) {
      // ensure the version folder is removed if it failed
      await onRemoveVersion(version);
      releases = releases.map((release) =>
        release.version === version
          ? { ...release, pendingAction: false }
          : release,
      );
      return;
    }

    versionState[source.activeKey] = version;
    const isLatest = version === releases[0]?.version;

    if ($UpdateStore[source.updateKey].updateAvailable && isLatest) {
      $UpdateStore[source.updateKey].updateAvailable = false;
    }

    releases = releases.map((release) =>
      release.version === version
        ? { ...release, pendingAction: false, isDownloaded: true }
        : release,
    );
    await saveOfficialVersionChange(version);
  }

  async function onRemoveVersion(version: string) {
    // Mark that release as being removed
    releases = releases.map((release) =>
      release.version === version
        ? { ...release, pendingAction: true }
        : release,
    );
    const error = await source.remove(version);
    if (error) {
      releases = releases.map((release) =>
        release.version === version
          ? { ...release, pendingAction: false }
          : release,
      );
      return;
    }

    // Update the store, if we removed the active version
    if (versionState[source.activeKey] === version) {
      versionState[source.activeKey] = undefined;
    }

    // Then mark it as not downloaded
    releases = releases.map((release) =>
      release.version === version
        ? { ...release, pendingAction: false, isDownloaded: false }
        : release,
    );
  }

  const isPending = $derived(releases.some((r) => r.pendingAction));
</script>

<p class="text-sm mt-2 mb-2">{$_(source.header)}</p>
{#if loading}
  <div class="flex flex-col justify-center items-center">
    <Spinner color="yellow" size="12" />
  </div>
{:else if releases.length > 0}
  <div class="flex items-center mb-2">
    <div class="grow">
      <p class="text-sm text-gray-300">
        {$_(source.description)}
      </p>
    </div>
    <div class="flex">
      <Button
        class="p-2! mr-2 rounded-md bg-orange-500 hover:bg-orange-600 text-slate-900"
        onclick={refreshVersionList}
        disabled={isPending}
      >
        <IconRefresh
          aria-label={$_("settings_versions_icon_refresh_altText")}
        />
      </Button>
      <Button
        class="p-2! rounded-md bg-orange-500 hover:bg-orange-600 text-slate-900"
        onclick={async () => openPath(config?.installationDir + source.folder)}
      >
        <IconFolderOpen
          aria-label={$_("settings_versions_icon_openFolder_altText")}
        />
      </Button>
    </div>
  </div>

  <Latest
    latest={releases[0]}
    onVersionChange={saveOfficialVersionChange}
    {onRemoveVersion}
    {onDownloadVersion}
    {isPending}
    activeKey={source.activeKey}
  ></Latest>

  <VersionList
    releaseList={releases.slice(1)}
    onVersionChange={saveOfficialVersionChange}
    {onRemoveVersion}
    {onDownloadVersion}
    {isPending}
    activeKey={source.activeKey}
  />
{:else}
  <Alert class="bg-slate-900 grow text-red-400">
    {$_("settings_versions_noReleasesFound")}
  </Alert>
{/if}
