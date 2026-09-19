<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { Alert, Badge, Button, Spinner } from "flowbite-svelte";
  import IconArrowLeft from "~icons/mdi/arrow-left";
  import IconCheck from "~icons/mdi/check";
  import IconPalette from "~icons/mdi/palette";
  import IconDownload from "~icons/mdi/tray-arrow-down";
  import IconFolder from "~icons/mdi/folder-open";
  import IconInformation from "~icons/mdi/information-outline";
  import { navigate, route } from "/src/router";
  import type { SupportedGame } from "$lib/rpc/bindings/SupportedGame";
  import { toSupportedGame } from "$lib/rpc/SupportedGame";
  import type { ModInfo } from "$lib/rpc/bindings/ModInfo";
  import type { ModSourceData } from "$lib/rpc/bindings/ModSourceData";
  import { getModInfo } from "$lib/rpc/ModInfo";
  import { getModSourcesData } from "$lib/rpc/cache";
  import {
    downloadAndExtractTexturePack,
    extractNewTexturePack,
    listExtractedTexturePackInfo,
  } from "$lib/rpc/features";
  import { filePrompt } from "$lib/utils/file-dialogs";
  import { platform } from "@tauri-apps/plugin-os";
  import { asJobType } from "$lib/job/jobs";
  import { config } from "/src/state/config.svelte";
  import placeholder from "$assets/images/mod-thumbnail-placeholder.webp";

  // Route parameters
  const gameParam = $derived(route.params.game_name);
  const modSource = $derived(route.params.source_name);
  const modName = $derived(route.params.mod_name);

  let activeGame: SupportedGame | undefined = $state(undefined);

  $effect(() => {
    const activeGameFromParam = toSupportedGame(gameParam);
    if (activeGameFromParam) {
      activeGame = activeGameFromParam;
    }
  });

  interface AffiliatedPackView {
    key: string;
    displayName: string;
    description: string;
    authors: string[];
    tags: string[];
    version: string;
    coverArtUrl: string | null;
    downloadUrl?: string;
    isDownloaded: boolean;
    isEnabled: boolean;
    isAffiliated: boolean;
  }

  let loading = $state(true);
  let applying = $state(false);
  let addingLocal = $state(false);
  let modInfo: ModInfo | undefined = $state(undefined);
  let affiliatedPacks: AffiliatedPackView[] = $state([]);
  let initialEnabledState: Record<string, boolean> = $state({});

  // Computes whether there are unapplied changes pending
  const hasChanges = $derived.by(() => {
    for (const pack of affiliatedPacks) {
      if (initialEnabledState[pack.key] !== pack.isEnabled) {
        return true;
      }
    }
    return false;
  });

  onMount(async () => {
    await loadModTexturePacks();
  });

  // Loads affiliated texture packs and correlated disk/config states
  async function loadModTexturePacks() {
    if (!activeGame || !modSource || !modName) {
      return;
    }
    loading = true;

    try {
      modInfo = await getModInfo(activeGame, modName, modSource);
      const allSources = await getModSourcesData();
      const sourceData: ModSourceData | undefined = Object.values(
        allSources,
      ).find((s) => s.sourceName === modSource);

      const currentlyEnabledPacks: string[] =
        config?.games?.[activeGame]?.mods?.[modSource]?.[modName]
          ?.texturePacks ?? [];

      const extractedInfo = await listExtractedTexturePackInfo(activeGame);

      // Determine release base URL if available for release-matching
      let releaseBaseUrls: string[] = [];
      if (modInfo?.versions) {
        for (const v of modInfo.versions) {
          for (const assetUrl of Object.values(v.assets || {})) {
            if (typeof assetUrl === "string" && assetUrl.includes("/")) {
              const lastSlash = assetUrl.lastIndexOf("/");
              if (lastSlash !== -1) {
                releaseBaseUrls.push(assetUrl.substring(0, lastSlash + 1));
              }
            }
          }
        }
      }

      const packs: AffiliatedPackView[] = [];
      const seenKeys = new Set<string>();

      // Scan sourceData.texturePacks for affiliated packs
      if (sourceData?.texturePacks) {
        for (const [packKey, pack] of Object.entries(sourceData.texturePacks)) {
          if (
            pack.supportedGames &&
            pack.supportedGames.length > 0 &&
            !pack.supportedGames.includes(activeGame)
          ) {
            continue;
          }

          let isAttached = false;

          // Check release base URL match
          if (releaseBaseUrls.length > 0 && pack.versions) {
            isAttached = pack.versions.some((v) =>
              Object.values(v.assets || {}).some(
                (url) =>
                  url &&
                  typeof url === "string" &&
                  releaseBaseUrls.some((base) => url.startsWith(base)),
              ),
            );
          }

          // Check mod name in tags
          if (!isAttached && pack.tags) {
            const targetTag = modName.toLowerCase();
            isAttached = pack.tags.some(
              (t) =>
                t.toLowerCase() === targetTag ||
                t.toLowerCase() === `mod:${targetTag}`,
            );
          }

          // Check key / slug match
          if (!isAttached) {
            const lowerKey = packKey.toLowerCase();
            const lowerMod = modName.toLowerCase();
            isAttached =
              lowerKey === lowerMod ||
              lowerKey.startsWith(`${lowerMod}-`) ||
              lowerKey.startsWith(`${lowerMod}_`) ||
              lowerMod.startsWith(`${lowerKey}-`);
          }

          if (isAttached) {
            seenKeys.add(packKey);

            // Select matching version
            const selectedVersion =
              pack.versions?.find((v) =>
                releaseBaseUrls.some((base) =>
                  Object.values(v.assets || {}).some(
                    (u) => typeof u === "string" && u.startsWith(base),
                  ),
                ),
              ) ||
              pack.versions?.[0] ||
              undefined;

            // Resolve platform download URL
            const plat = platform();
            let downloadUrl: string | undefined = undefined;
            if (selectedVersion?.assets) {
              if (selectedVersion.assets[plat]) {
                downloadUrl = selectedVersion.assets[plat]!;
              } else if (selectedVersion.assets["all"]) {
                downloadUrl = selectedVersion.assets["all"]!;
              } else if (selectedVersion.assets["windows"]) {
                downloadUrl = selectedVersion.assets["windows"]!;
              } else {
                const first = Object.values(selectedVersion.assets).find(
                  (u) => typeof u === "string" && u.length > 0,
                );
                if (first) {
                  downloadUrl = first;
                }
              }
            }

            const isDownloaded = Boolean(
              extractedInfo && packKey in extractedInfo,
            );
            const isEnabled = currentlyEnabledPacks.includes(packKey);

            packs.push({
              key: packKey,
              displayName: pack.displayName || packKey,
              description: pack.description || "",
              authors: pack.authors || [],
              tags: pack.tags || [],
              version: selectedVersion?.version || "1.0.0",
              coverArtUrl: pack.coverArtUrl || pack.thumbnailArtUrl || null,
              downloadUrl,
              isDownloaded,
              isEnabled,
              isAffiliated: true,
            });
          }
        }
      }

      // Also include any currently enabled pack that was not found in source catalog
      for (const enabledKey of currentlyEnabledPacks) {
        if (!seenKeys.has(enabledKey)) {
          seenKeys.add(enabledKey);
          const meta = extractedInfo?.[enabledKey];
          packs.push({
            key: enabledKey,
            displayName: meta?.name || enabledKey,
            description: meta?.description || "",
            authors: meta?.author ? [meta.author] : [],
            tags: meta?.tags || [],
            version: meta?.version || "1.0.0",
            coverArtUrl: null,
            downloadUrl: undefined,
            isDownloaded: Boolean(extractedInfo && enabledKey in extractedInfo),
            isEnabled: true,
            isAffiliated: false,
          });
        }
      }

      affiliatedPacks = packs;
      const initialMap: Record<string, boolean> = {};
      for (const p of packs) {
        initialMap[p.key] = p.isEnabled;
      }
      initialEnabledState = initialMap;
    } finally {
      loading = false;
    }
  }

  // Toggles the enabled state of a texture pack for this mod
  function togglePackEnabled(packKey: string) {
    const pack = affiliatedPacks.find((p) => p.key === packKey);
    if (!pack) return;
    pack.isEnabled = !pack.isEnabled;
    affiliatedPacks = [...affiliatedPacks];
  }

  // Applies selected texture packs exclusively to this mod
  async function applyChanges() {
    if (!activeGame || !modSource || !modName) {
      return;
    }
    applying = true;

    try {
      // Download any enabled packs that are not yet downloaded on disk
      for (const pack of affiliatedPacks) {
        if (pack.isEnabled && !pack.isDownloaded && pack.downloadUrl) {
          const err = await downloadAndExtractTexturePack(
            activeGame,
            pack.downloadUrl,
            pack.key,
          );
          if (err) {
            console.error(
              `Failed to download texture pack ${pack.key}: ${err}`,
            );
          }
        }
      }

      const enabledPacks = affiliatedPacks
        .filter((p) => p.isEnabled)
        .map((p) => p.key);

      navigate("/job/:job_type", {
        params: {
          job_type: asJobType("applyTexturePacks"),
        },
        search: {
          activeGame: activeGame,
          enabledPacks: JSON.stringify(enabledPacks),
          packsToDelete: JSON.stringify([]),
          modSourceName: modSource,
          modName: modName,
          returnTo: route.pathname,
        },
      });
    } finally {
      applying = false;
    }
  }

  // Prompts user for a local texture pack ZIP and extracts it
  async function addLocalTexturePack() {
    if (!activeGame) return;
    const path = await filePrompt(
      ["zip"],
      "ZIP",
      $_("features_modTextures_selectPrompt"),
    );
    if (!path) return;

    addingLocal = true;
    try {
      const err = await extractNewTexturePack(activeGame, path);
      if (!err) {
        await loadModTexturePacks();
      }
    } finally {
      addingLocal = false;
    }
  }

  // Returns back to the mod's overview page
  function goBackToMod() {
    if (activeGame && modSource && modName) {
      navigate(`/:game_name/mods/:source_name/:mod_name`, {
        params: {
          game_name: activeGame,
          source_name: modSource,
          mod_name: modName,
        },
      });
    }
  }
</script>

<div
  class="flex flex-col min-h-full flex-1 bg-[#141414] text-gray-200 p-6 gap-6"
>
  <!-- Top Navigation & Actions Bar -->
  <div
    class="flex flex-row items-center justify-between gap-4 border-b border-zinc-800 pb-4"
  >
    <div class="flex items-center gap-3">
      <Button
        outline
        class="border-solid border-zinc-700 rounded text-gray-300 hover:text-white hover:bg-zinc-800 font-medium px-3 py-2"
        onclick={goBackToMod}
        aria-label={$_("features_modTextures_backToMod")}
      >
        <IconArrowLeft class="text-lg" />
      </Button>
      <div>
        <div class="flex items-center gap-2">
          <h1
            class="text-xl font-bold tracking-tight text-white flex items-center gap-2"
          >
            <IconPalette class="text-orange-400" />
            {$_("features_modTextures_title")}
          </h1>
          {#if modName}
            <span
              class="text-xs bg-orange-950/80 text-orange-400 border border-orange-800/80 rounded px-2.5 py-0.5 font-semibold"
            >
              Mod: {modInfo?.displayName || modName}
            </span>
          {/if}
        </div>
        <p class="text-xs text-gray-400 mt-0.5">
          {$_("features_modTextures_subtitle")}
        </p>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center gap-2">
      <Button
        outline
        disabled={addingLocal || applying}
        class="border-solid border-zinc-700 text-gray-300 hover:text-white hover:bg-zinc-800 font-medium text-xs px-3 py-2 rounded flex items-center gap-1.5"
        onclick={addLocalTexturePack}
      >
        {#if addingLocal}
          <Spinner size="4" color="yellow" class="mr-1" />
        {:else}
          <IconFolder />
        {/if}
        {$_("features_modTextures_importLocal")}
      </Button>

      <Button
        disabled={!hasChanges || applying}
        class={`font-semibold text-xs px-4 py-2 rounded transition-all flex items-center gap-1.5 ${
          hasChanges
            ? "bg-green-500 hover:bg-green-600 text-slate-950 shadow-[0_0_12px_rgba(34,197,94,0.4)]"
            : "bg-zinc-800 text-gray-500 border border-zinc-700 cursor-not-allowed"
        }`}
        onclick={applyChanges}
      >
        {#if applying}
          <Spinner size="4" color="yellow" class="mr-1" />
          {$_("features_modTextures_applying")}
        {:else}
          <IconCheck />
          {$_("features_textures_applyChanges")}
        {/if}
      </Button>
    </div>
  </div>

  <!-- Main Body Content -->
  {#if loading}
    <div class="flex flex-col items-center justify-center py-20 gap-3">
      <Spinner color="yellow" size="12" />
      <span class="text-sm text-gray-400 font-medium"
        >{$_("features_modTextures_loading")}</span
      >
    </div>
  {:else if affiliatedPacks.length === 0}
    <!-- Empty State -->
    <div
      class="flex flex-col items-center justify-center rounded-xl border border-zinc-800 bg-zinc-900/40 p-12 text-center my-6"
    >
      <div
        class="w-16 h-16 rounded-full bg-zinc-800/80 flex items-center justify-center text-orange-400 text-2xl mb-4 border border-zinc-700"
      >
        <IconPalette />
      </div>
      <h2 class="text-lg font-bold text-white mb-1">
        {$_("features_modTextures_emptyTitle")}
      </h2>
      <p class="text-sm text-gray-400 max-w-md mb-6 leading-relaxed">
        {$_("features_modTextures_emptyDescription")}
      </p>
      <Button
        class="bg-orange-500 hover:bg-orange-600 text-slate-950 font-semibold px-4 py-2 rounded flex items-center gap-2 text-sm"
        onclick={addLocalTexturePack}
      >
        <IconFolder />
        {$_("features_modTextures_importZipButton")}
      </Button>
    </div>
  {:else}
    <!-- Texture Packs List -->
    <div class="flex flex-col gap-4">
      <div class="flex items-center justify-between text-xs text-gray-400 px-1">
        <span
          >{affiliatedPacks.length}
          {$_("features_modTextures_availableCount")}</span
        >
        {#if hasChanges}
          <span class="text-amber-400 font-semibold flex items-center gap-1">
            <IconInformation />
            {$_("features_modTextures_pendingChanges")}
          </span>
        {/if}
      </div>

      <div class="grid grid-cols-1 gap-4">
        {#each affiliatedPacks as pack (pack.key)}
          <div
            class={`flex flex-col md:flex-row gap-5 p-5 rounded-xl border transition-all duration-200 ${
              pack.isEnabled
                ? "border-green-500/50 bg-green-950/10 shadow-[0_4px_20px_rgba(34,197,94,0.08)]"
                : "border-zinc-800 bg-zinc-900/50 hover:border-zinc-700"
            }`}
          >
            <!-- Thumbnail / Cover Art -->
            <div
              class="relative w-full md:w-48 h-32 shrink-0 rounded-lg overflow-hidden border border-zinc-800 bg-zinc-950 flex items-center justify-center"
            >
              <img
                src={pack.coverArtUrl || placeholder}
                alt={pack.displayName}
                class="w-full h-full object-cover"
                onerror={(e) => {
                  (e.currentTarget as HTMLImageElement).src = placeholder;
                }}
              />
              {#if pack.isEnabled}
                <div
                  class="absolute top-2 left-2 bg-green-500 text-slate-950 font-bold text-[10px] px-2 py-0.5 rounded-full flex items-center gap-1 shadow-md"
                >
                  <IconCheck />
                  {$_("features_modTextures_badgeActive")}
                </div>
              {/if}
            </div>

            <!-- Details Section -->
            <div class="flex flex-col flex-1 justify-between gap-3">
              <div>
                <div class="flex flex-wrap items-center gap-2 mb-1">
                  <h3 class="text-base font-bold text-white">
                    {pack.displayName}
                  </h3>
                  <span
                    class="text-xs bg-zinc-800 text-zinc-300 px-2 py-0.5 rounded font-mono border border-zinc-700"
                  >
                    v{pack.version}
                  </span>
                  {#if pack.isAffiliated}
                    <span
                      class="text-[11px] bg-blue-950 text-blue-400 border border-blue-800/80 px-2 py-0.5 rounded font-semibold"
                    >
                      {$_("features_modTextures_badgeAffiliated")}
                    </span>
                  {/if}
                  {#if pack.isDownloaded}
                    <span
                      class="text-[11px] bg-emerald-950 text-emerald-400 border border-emerald-800/80 px-2 py-0.5 rounded font-semibold"
                    >
                      {$_("features_modTextures_badgeDownloaded")}
                    </span>
                  {:else}
                    <span
                      class="text-[11px] bg-amber-950 text-amber-400 border border-amber-800/80 px-2 py-0.5 rounded font-semibold flex items-center gap-1"
                    >
                      <IconDownload />
                      {$_("features_modTextures_badgeOnline")}
                    </span>
                  {/if}
                </div>

                {#if pack.authors && pack.authors.length > 0}
                  <p class="text-xs text-gray-400 mb-2">
                    {$_("features_modTextures_byAuthor")}
                    <span class="text-gray-300 font-medium"
                      >{pack.authors.join(", ")}</span
                    >
                  </p>
                {/if}

                {#if pack.description}
                  <p class="text-xs text-gray-300 leading-relaxed max-w-2xl">
                    {pack.description}
                  </p>
                {/if}

                <!-- Tags -->
                {#if pack.tags && pack.tags.length > 0}
                  <div class="flex flex-wrap gap-1.5 mt-3">
                    {#each pack.tags as tag}
                      <span
                        class="text-[10px] bg-zinc-800/80 text-zinc-400 px-2 py-0.5 rounded border border-zinc-700/60 font-medium"
                      >
                        #{tag}
                      </span>
                    {/each}
                  </div>
                {/if}
              </div>

              <!-- Toggle Control -->
              <div
                class="flex items-center justify-between border-t border-zinc-800/60 pt-3"
              >
                <div class="text-xs">
                  {#if pack.isEnabled}
                    <span class="text-green-400 font-medium"
                      >{$_("features_modTextures_packActiveDescription")}</span
                    >
                  {:else}
                    <span class="text-gray-500"
                      >{$_(
                        "features_modTextures_packInactiveDescription",
                      )}</span
                    >
                  {/if}
                </div>

                <Button
                  size="xs"
                  class={`px-3 py-1.5 rounded font-semibold text-xs transition-all flex items-center gap-1.5 ${
                    pack.isEnabled
                      ? "bg-red-950/40 text-red-400 border border-red-800/60 hover:bg-red-900/60"
                      : "bg-orange-500 hover:bg-orange-600 text-slate-950 shadow-sm"
                  }`}
                  onclick={() => togglePackEnabled(pack.key)}
                >
                  {#if pack.isEnabled}
                    {$_("features_modTextures_disable")}
                  {:else}
                    <IconCheck />
                    {$_("features_modTextures_enable")}
                  {/if}
                </Button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Isolation Guarantee Note -->
  <Alert
    class="bg-zinc-900/60 border border-zinc-800 text-gray-400 text-xs mt-auto"
  >
    <span class="font-bold text-orange-400 mr-1"
      >{$_("features_modTextures_isolationReminderTitle")}</span
    >
    {$_("features_modTextures_isolationReminderText")}
  </Alert>
</div>
