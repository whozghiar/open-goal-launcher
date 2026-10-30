<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import logo from "$assets/images/icon.webp";
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import IconWindowMinimize from "~icons/mdi/window-minimize";
  import IconHelpCircle from "~icons/mdi/help-circle";
  import IconWindowClose from "~icons/mdi/window-close";
  import IconDownload from "~icons/mdi/tray-arrow-down";
  import { UpdateStore, type UpdateKey } from "$lib/stores/AppStore";
  import { isInDebugMode } from "$lib/utils/common";
  import {
    downloadOfficialVersion,
    listDownloadedAnimationStudioVersions,
    listDownloadedLevelEditorVersions,
    listDownloadedVersions,
    removeOldVersions,
  } from "$lib/rpc/versions";
  import {
    getLatestAnimationStudioRelease,
    getLatestLevelEditorRelease,
    getLatestOfficialRelease,
    type ReleaseInfo,
  } from "$lib/utils/github";
  import { _ } from "svelte-i18n";
  import { toastStore } from "$lib/stores/ToastStore";
  import { setActiveVersion } from "$lib/rpc/config";
  import { check } from "@tauri-apps/plugin-updater";
  import {
    versionState,
    type ActiveVersionKey,
  } from "/src/state/VersionState.svelte";
  import { navigate } from "/src/router";
  import { route } from "../../router";
  import { Tooltip } from "flowbite-svelte";
  import { config } from "/src/state/config.svelte";

  let launcherVersion: string | null = $state(null);
  let launcherUpdateAvailable = $state(false);
  let checkedToolingUpdate = $state(false);
  const appWindow = getCurrentWebviewWindow();

  async function downloadLatestVersion(version: string, url: string) {
    await downloadOfficialVersion(version, url);
    $UpdateStore.selectedTooling.updateAvailable = false;
    await saveOfficialVersionChange(version);
  }

  async function saveOfficialVersionChange(version: string) {
    const success = await setActiveVersion(version);
    if (success) {
      versionState.activeToolingVersion = version;
      toastStore.makeToast($_("toasts_savedToolingVersion"), "info");
    }
  }

  onMount(async () => {
    // Get current versions
    const appVersion = await getVersion();
    launcherVersion = appVersion.includes("whozghiar")
      ? `v${appVersion}`
      : `v${appVersion}-whozghiar`;

    // Check for a launcher update
    if (!isInDebugMode()) {
      const update = await check();
      if (update) {
        launcherUpdateAvailable = true;
      }
    }
  });

  $effect(() => {
    if (checkedToolingUpdate) return;
    if (route.pathname === "/startup") return;
    checkedToolingUpdate = true;
    checkForToolingUpdate();
    checkForProgramUpdate(
      "activeLevelEditorVersion",
      "selectedLevelEditor",
      config?.levelEditorVersion,
      getLatestLevelEditorRelease,
      listDownloadedLevelEditorVersions,
    );
    checkForProgramUpdate(
      "activeAnimationStudioVersion",
      "selectedAnimationStudio",
      config?.animationStudioVersion,
      getLatestAnimationStudioRelease,
      listDownloadedAnimationStudioVersions,
    );
  });

  async function checkForToolingUpdate() {
    const latestToolingVersion = await getLatestOfficialRelease();
    const latestVersion = latestToolingVersion?.version;

    if (!latestVersion) return;
    if (versionState.activeToolingVersion === latestVersion) return;

    // Check that we havn't already downloaded it
    const downloadedOfficialVersions = await listDownloadedVersions();
    if (downloadedOfficialVersions.includes(latestVersion)) return;

    $UpdateStore.selectedTooling = {
      updateAvailable: true,
      versionNumber: latestVersion,
    };
  }

  // a program the launcher installs per version: Jak Level Editor, Jak Animation Studio
  async function checkForProgramUpdate(
    activeKey: ActiveVersionKey,
    updateKey: UpdateKey,
    active: string | null | undefined,
    getLatestRelease: () => Promise<ReleaseInfo | undefined>,
    listDownloaded: () => Promise<string[]>,
  ) {
    versionState[activeKey] = active;
    const latestRelease = await getLatestRelease();
    const latestVersion = latestRelease?.version;

    // a release without the portable build for this system offers nothing to download
    if (!latestVersion || !latestRelease.downloadUrl) return;
    if (versionState[activeKey] === latestVersion) return;

    const downloadedVersions = await listDownloaded();
    if (downloadedVersions.includes(latestVersion)) return;

    $UpdateStore[updateKey] = {
      updateAvailable: true,
      versionNumber: latestVersion,
    };
  }

  // TODO: handle this in the new startup route
  // download the latest `jak-project` binaries
  async function autoUpdateTooling() {
    const shouldAutoUpdate = config?.autoUpdateGames;
    if (!shouldAutoUpdate) return;

    const latestToolingVersion = await getLatestOfficialRelease();
    const latestVersion = latestToolingVersion?.version;
    const url = latestToolingVersion?.downloadUrl;

    if (!latestVersion) return;
    if (!url) return;

    await downloadLatestVersion(latestVersion, url);
    // delete the existing `jak-project` binaries
    await removeOldVersions();
  }
</script>

{#snippet programVersion(
  label: string,
  version: string | null | undefined,
  updateAvailable: boolean,
  settings: string,
)}
  <div class="h-6 shrink-0 border-l border-neutral-800"></div>

  <div class="flex items-center gap-1.5 font-mono text-sm whitespace-nowrap">
    <span class="text-neutral-400">{label}</span>
    <span class="text-neutral-500">
      {version ?? $_("header_toolingNotSet")}
    </span>

    {#if updateAvailable}
      <a
        class="pointer-events-auto text-green-500 hover:text-green-300 animate-pulse relative"
        href={settings}
      >
        <IconDownload />
      </a>
      <Tooltip type="auto" trigger="hover">
        {$_("header_updateAvailable")}
      </Tooltip>
    {/if}
  </div>
{/snippet}

<header
  class="flex shrink-0 h-8 items-center bg-[#101010] pl-2 z-5"
  data-tauri-drag-region
>
  <div class="flex shrink-0 items-center gap-2 pointer-events-none">
    <img
      class="h-6"
      src={logo}
      alt="OpenGOAL logo"
      aria-label="OpenGOAL logo"
    />
    <p class="text-base font-black tracking-tight text-white">OpenGOAL</p>
  </div>

  <div class="mx-2 h-6 shrink-0 border-l border-neutral-800"></div>

  <div class="flex min-w-0 items-center gap-1.5 pointer-events-none">
    <div class="flex items-center gap-1.5 font-mono text-sm whitespace-nowrap">
      <span class="text-neutral-400">{$_("header_launcherVersionLabel")}</span>
      <span class="text-neutral-500">{launcherVersion}</span>
      {#if launcherUpdateAvailable}
        <a
          class="pointer-events-auto text-green-500 hover:text-green-300 animate-pulse relative"
          href="/update/launcher"
        >
          <IconDownload />
        </a>
        <Tooltip type="auto" trigger="hover">
          {$_("header_updateAvailable")}
        </Tooltip>
      {/if}
    </div>

    <div class="h-6 shrink-0 border-l border-neutral-800"></div>

    <div class="flex items-center gap-1.5 font-mono text-sm whitespace-nowrap">
      <span class="text-neutral-400">
        {#if versionState.displayModVersion}
          {$_("header_modVersionLabel")}
        {:else}
          {$_("header_toolingVersionLabel")}
        {/if}
      </span>

      <span class="text-neutral-500">
        {#if versionState.displayModVersion}
          {#if !versionState.activeModVersionInfo.installed}
            {$_("header_modNotInstalled")}
          {:else if versionState.activeModVersionInfo.installedVersion}
            {versionState.activeModVersionInfo.installedVersion}
          {:else}
            <span>&nbsp;</span>
          {/if}
        {:else}
          {!versionState.activeToolingVersion
            ? $_("header_toolingNotSet")
            : versionState.activeToolingVersion}
        {/if}
      </span>

      {#if $UpdateStore.selectedTooling.updateAvailable && !versionState.displayModVersion}
        <a
          class="pointer-events-auto text-green-500 hover:text-green-300 animate-pulse relative"
          href="/settings/versions"
        >
          <IconDownload />
        </a>
        <Tooltip type="auto" trigger="hover">
          {$_("header_updateAvailable")}
        </Tooltip>
      {/if}
    </div>

    {@render programVersion(
      $_("header_levelEditorVersionLabel"),
      versionState.activeLevelEditorVersion,
      $UpdateStore.selectedLevelEditor.updateAvailable,
      "/settings/level-editor",
    )}
    {@render programVersion(
      $_("header_animationStudioVersionLabel"),
      versionState.activeAnimationStudioVersion,
      $UpdateStore.selectedAnimationStudio.updateAvailable,
      "/settings/animation-studio",
    )}
  </div>

  <!-- above the versions, which a narrow window does not fit -->
  <div class="relative ml-auto flex shrink-0 self-stretch bg-[#101010]">
    <button
      class="flex h-full w-12 items-center justify-center text-neutral-200 hover:bg-white/10"
      onclick={() => navigate("/help")}
      aria-label="Help"
    >
      <IconHelpCircle />
    </button>
    <Tooltip placement="bottom" type="auto" trigger="hover"
      >{$_("sidebar_help")}</Tooltip
    >

    <div class="h-6 my-auto shrink-0 border-l border-neutral-800"></div>

    <button
      class="flex h-full w-12 items-center justify-center hover:bg-white/10"
      onclick={() => appWindow.minimize()}
      aria-label="Minimize"
    >
      <IconWindowMinimize />
    </button>

    <button
      class="flex h-full w-12 items-center justify-center hover:bg-red-600"
      onclick={() => appWindow.close()}
      aria-label="Close"
    >
      <IconWindowClose />
    </button>
  </div>
</header>
