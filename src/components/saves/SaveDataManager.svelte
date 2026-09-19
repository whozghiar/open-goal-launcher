<script lang="ts">
  import IconArrowLeft from "~icons/mdi/arrow-left";
  import IconFolder from "~icons/mdi/folder-open";
  import IconContentCopy from "~icons/mdi/content-copy";
  import IconDelete from "~icons/mdi/delete";
  import IconBackup from "~icons/mdi/backup-restore";
  import IconRefresh from "~icons/mdi/refresh";
  import {
    Alert,
    Badge,
    Button,
    Modal,
    Spinner,
    Tooltip,
  } from "flowbite-svelte";
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { navigate, route } from "/src/router";
  import type { SupportedGame } from "$lib/rpc/bindings/SupportedGame";
  import { toSupportedGame } from "$lib/rpc/SupportedGame";
  import { toastStore } from "$lib/stores/ToastStore";
  import {
    listGameSaveInstalls,
    copySave,
    moveSave,
    backupSave,
    deleteSave,
    openSaveFolder,
  } from "$lib/rpc/saves";
  import type { SaveInstallInfo } from "$lib/rpc/bindings/SaveInstallInfo";
  import type { SaveFolderInfo } from "$lib/rpc/bindings/SaveFolderInfo";
  import type { SaveSlotInfo } from "$lib/rpc/bindings/SaveSlotInfo";

  const GAMES: { id: SupportedGame; label: string }[] = [
    { id: "jak1", label: "Jak & Daxter" },
    { id: "jak2", label: "Jak II" },
    { id: "jak3", label: "Jak 3" },
    { id: "jakx", label: "Jak X" },
  ];

  let currentGame: SupportedGame = $state(
    toSupportedGame(route.params.game_name) ?? "jak1",
  );

  let loaded = $state(false);
  let loadingInstalls = $state(false);
  let installs: SaveInstallInfo[] = $state([]);
  let selectedInstallId: string = $state("vanilla");
  let selectedFolderName: string | null = $state(null);
  let lastFetchedGame: SupportedGame | undefined = $state(undefined);

  $effect(() => {
    const routeGame = toSupportedGame(route.params.game_name);
    if (routeGame && routeGame !== currentGame) {
      currentGame = routeGame;
    }
  });

  $effect(() => {
    if (route.params.mod_name && route.params.source_name) {
      const modInstallId = `mod:${route.params.source_name}:${route.params.mod_name}`;
      if (installs.some((i) => i.id === modInstallId)) {
        selectedInstallId = modInstallId;
      }
    }
  });

  $effect(() => {
    if (currentGame && currentGame !== lastFetchedGame) {
      lastFetchedGame = currentGame;
      refreshInstalls(currentGame);
    }
  });

  let activeInstall = $derived(
    installs.find((i) => i.id === selectedInstallId) ?? installs[0],
  );

  let availableFolders: SaveFolderInfo[] = $derived(
    activeInstall?.folders ?? [],
  );

  $effect(() => {
    if (
      availableFolders.length === 1 &&
      availableFolders[0].folderName === "default"
    ) {
      selectedFolderName = "default";
    } else if (
      availableFolders.length > 0 &&
      selectedFolderName !== null &&
      !availableFolders.some((f) => f.folderName === selectedFolderName)
    ) {
      selectedFolderName = null;
    }
  });

  let activeFolder: SaveFolderInfo | null = $derived(
    availableFolders.find((f) => f.folderName === selectedFolderName) ?? null,
  );

  let currentFolderSaves: SaveSlotInfo[] = $derived(
    activeFolder ? activeFolder.saves : (activeInstall?.saves ?? []),
  );

  let showTransferModal = $state(false);
  let transferSourceSave: SaveSlotInfo | null = $state(null);
  let targetInstallId: string = $state("");
  let targetFolderName: string = $state("default");
  let targetSlotNumber: number = $state(0);
  let isMoveOperation: boolean = $state(false);
  let performingAction = $state(false);

  let showDeleteModal = $state(false);
  let saveToDelete: SaveSlotInfo | null = $state(null);

  let targetInstall = $derived(
    installs.find((i) => i.id === targetInstallId) ?? null,
  );

  let targetSlotOccupied = $derived.by(() => {
    if (!targetInstall) return false;
    return targetInstall.saves.some(
      (s) =>
        s.slotNumber === targetSlotNumber &&
        (targetFolderName === "default" || s.folderName === targetFolderName),
    );
  });

  onMount(() => {
    lastFetchedGame = currentGame;
    refreshInstalls(currentGame);
  });

  function switchGame(gameId: SupportedGame) {
    if (currentGame === gameId) return;
    currentGame = gameId;
    selectedFolderName = null;
    selectedInstallId = "vanilla";
    navigate(`/:game_name/saves`, { params: { game_name: gameId } });
    refreshInstalls(gameId);
  }

  async function refreshInstalls(game = currentGame) {
    if (!game) {
      loaded = true;
      return;
    }
    loadingInstalls = true;
    try {
      const result = await listGameSaveInstalls(game);
      if (Array.isArray(result)) {
        installs = result;
        if (
          !installs.some((i) => i.id === selectedInstallId) &&
          installs.length > 0
        ) {
          selectedInstallId = installs[0].id;
        }
      } else {
        installs = [];
      }
    } catch (err) {
      toastStore.makeToast(`Failed to load saves: ${err}`, "error");
      installs = [];
    } finally {
      loadingInstalls = false;
      loaded = true;
    }
  }

  function openTransferModal(save: SaveSlotInfo) {
    transferSourceSave = save;
    const otherInstalls = installs.filter((i) => i.id !== selectedInstallId);
    targetInstallId = otherInstalls.length > 0 ? otherInstalls[0].id : "";
    targetFolderName = save.folderName ?? "default";
    targetSlotNumber = save.slotNumber ?? 0;
    isMoveOperation = false;
    showTransferModal = true;
  }

  async function executeTransfer() {
    if (!currentGame || !transferSourceSave || !targetInstallId) return;

    performingAction = true;
    try {
      const destFolder =
        targetFolderName && targetFolderName !== "default"
          ? targetFolderName
          : null;

      if (isMoveOperation) {
        await moveSave(
          currentGame,
          selectedInstallId,
          targetInstallId,
          transferSourceSave.fileName,
          destFolder,
          targetSlotNumber,
          true,
        );
        toastStore.makeToast($_("saveManager_moveSuccess"), "info");
      } else {
        await copySave(
          currentGame,
          selectedInstallId,
          targetInstallId,
          transferSourceSave.fileName,
          destFolder,
          targetSlotNumber,
          true,
        );
        toastStore.makeToast($_("saveManager_copySuccess"), "info");
      }
      showTransferModal = false;
      await refreshInstalls();
    } catch (err) {
      toastStore.makeToast(`Transfer failed: ${err}`, "error");
    } finally {
      performingAction = false;
    }
  }

  async function handleBackup(save: SaveSlotInfo) {
    if (!currentGame) return;
    performingAction = true;
    try {
      const backupName = await backupSave(
        currentGame,
        selectedInstallId,
        save.fileName,
      );
      toastStore.makeToast(
        `${$_("saveManager_backupSuccess")} (${backupName})`,
        "info",
      );
      await refreshInstalls();
    } catch (err) {
      toastStore.makeToast(`Backup failed: ${err}`, "error");
    } finally {
      performingAction = false;
    }
  }

  function openDeleteModal(save: SaveSlotInfo) {
    saveToDelete = save;
    showDeleteModal = true;
  }

  async function executeDelete() {
    if (!currentGame || !saveToDelete) return;
    performingAction = true;
    try {
      await deleteSave(currentGame, selectedInstallId, saveToDelete.fileName);
      toastStore.makeToast($_("saveManager_deleteSuccess"), "info");
      showDeleteModal = false;
      saveToDelete = null;
      await refreshInstalls();
    } catch (err) {
      toastStore.makeToast(`Delete failed: ${err}`, "error");
    } finally {
      performingAction = false;
    }
  }

  async function openCurrentSaveFolder() {
    if (!activeInstall?.saveDir) return;
    try {
      const subFolder =
        selectedFolderName && selectedFolderName !== "default"
          ? selectedFolderName
          : null;
      await openSaveFolder(currentGame, activeInstall.id, subFolder);
    } catch (err) {
      toastStore.makeToast(`Unable to open folder: ${err}`, "error");
    }
  }

  function goBack() {
    if (route.params.mod_name && route.params.source_name && currentGame) {
      navigate(`/:game_name/mods/:source_name/:mod_name`, {
        params: {
          game_name: currentGame,
          source_name: route.params.source_name,
          mod_name: route.params.mod_name,
        },
      });
    } else if (currentGame) {
      navigate(`/:game_name/`, { params: { game_name: currentGame } });
    } else {
      navigate("/");
    }
  }

  function formatBytes(bytes: bigint | number): string {
    const num = Number(bytes);
    if (num < 1024) return `${num} B`;
    return `${(num / 1024).toFixed(1)} KB`;
  }

  function formatDate(timestamp: bigint | number): string {
    const num = Number(timestamp);
    if (!num) return "Unknown";
    return new Date(num).toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  let slotList = $derived.by(() => {
    if (!activeInstall || !activeFolder) return [];
    const maxSlots = currentGame === "jak1" ? 4 : 8;
    const slots = [];
    const usedFiles = new Set<string>();

    for (let slotIdx = 0; slotIdx < maxSlots; slotIdx++) {
      const existing = currentFolderSaves.find(
        (s) => s.slotNumber === slotIdx && !usedFiles.has(s.fileName),
      );
      if (existing) {
        usedFiles.add(existing.fileName);
      }
      slots.push({
        id: `slot-${slotIdx}-${existing?.fileName ?? "empty"}`,
        slotNumber: slotIdx,
        save: existing ?? null,
      });
    }

    const remainingSaves = currentFolderSaves.filter(
      (s) => !usedFiles.has(s.fileName),
    );
    for (let idx = 0; idx < remainingSaves.length; idx++) {
      const extra = remainingSaves[idx];
      slots.push({
        id: `extra-${idx}-${extra.fileName}`,
        slotNumber: extra.slotNumber ?? -1,
        save: extra,
      });
    }
    return slots;
  });
</script>

<div class="flex flex-col min-h-full flex-1 bg-[#1e1e1e] p-5 gap-4 text-white">
  {#if !loaded}
    <div class="flex items-center justify-center h-64">
      <Spinner color="yellow" size="12" />
    </div>
  {:else}
    <!-- Top Action Bar -->
    <div
      class="flex flex-wrap items-center justify-between gap-3 border-b border-neutral-700/60 pb-3"
    >
      <div class="flex items-center gap-3">
        <Button
          id="btn-save-back"
          outline
          class="border-solid rounded text-white hover:dark:text-slate-900 hover:bg-white font-semibold p-2.5"
          onclick={goBack}
          aria-label={$_("saveManager_back")}
        >
          <IconArrowLeft class="w-5 h-5" />
        </Button>
        <Tooltip triggeredBy="#btn-save-back" placement="bottom" type="dark">
          {$_("saveManager_tooltip_back")}
        </Tooltip>

        <div>
          <h1 class="text-xl font-bold tracking-wide">
            {$_("saveManager_title")}
          </h1>
          <p class="text-xs text-neutral-400">
            {$_("saveManager_subtitle")}
          </p>
        </div>
      </div>

      <!-- Right Action Controls -->
      <div class="flex items-center gap-2">
        <Button
          id="btn-save-refresh"
          size="sm"
          outline
          class="border-neutral-600 text-neutral-200 hover:bg-neutral-800"
          onclick={() => refreshInstalls()}
          disabled={loadingInstalls}
        >
          <IconRefresh
            class="w-4 h-4 mr-1 {loadingInstalls ? 'animate-spin' : ''}"
          />
          {$_("saveManager_refresh")}
        </Button>
        <Tooltip triggeredBy="#btn-save-refresh" placement="bottom" type="dark">
          {$_("saveManager_tooltip_refresh")}
        </Tooltip>

        {#if activeInstall}
          <Button
            id="btn-save-open-folder"
            size="sm"
            class="bg-amber-500 hover:bg-amber-600 text-black font-semibold"
            onclick={openCurrentSaveFolder}
          >
            <IconFolder class="w-4 h-4 mr-1.5" />
            {$_("saveManager_openFolder")}
          </Button>
          <Tooltip
            triggeredBy="#btn-save-open-folder"
            placement="bottom"
            type="dark"
          >
            {$_("saveManager_tooltip_openFolder")}
          </Tooltip>
        {/if}
      </div>
    </div>

    <!-- Game Selector Bar -->
    <div
      class="flex items-center gap-3 bg-neutral-900/80 p-2.5 rounded-lg border border-neutral-800"
    >
      <span
        class="text-xs font-semibold text-neutral-400 uppercase tracking-wider px-2"
      >
        {$_("saveManager_game")}:
      </span>
      <div class="flex flex-wrap gap-2 flex-1">
        {#each GAMES as g (g.id)}
          <button
            type="button"
            class="px-3.5 py-1.5 rounded-md text-xs font-semibold transition-all flex items-center gap-1.5 {currentGame ===
            g.id
              ? 'bg-amber-500 text-black shadow font-bold'
              : 'bg-neutral-800 text-neutral-300 hover:bg-neutral-700'}"
            onclick={() => switchGame(g.id)}
          >
            <span>{g.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Installation Selector Bar -->
    <div
      class="flex items-center gap-3 bg-neutral-900/60 p-3 rounded-lg border border-neutral-800"
    >
      <span class="text-sm font-semibold text-neutral-300 whitespace-nowrap">
        {$_("saveManager_install")}:
      </span>
      <div class="flex flex-wrap gap-2 flex-1">
        {#each installs as inst (inst.id)}
          <button
            type="button"
            class="px-3 py-1.5 rounded-md text-sm font-medium transition-all flex items-center gap-2 {selectedInstallId ===
            inst.id
              ? 'bg-amber-500 text-black font-semibold shadow'
              : 'bg-neutral-800 text-neutral-300 hover:bg-neutral-700'}"
            onclick={() => {
              selectedInstallId = inst.id;
              selectedFolderName = null;
            }}
          >
            <span>{inst.name}</span>
            {#if inst.isVanilla}
              <Badge color="blue" class="text-xs px-1.5 py-0.5">
                {$_("saveManager_vanillaBadge")}
              </Badge>
            {:else}
              <Badge
                color="gray"
                class="text-xs px-1.5 py-0.5 bg-neutral-700 text-neutral-200"
              >
                {$_("saveManager_modBadge")}
              </Badge>
            {/if}
          </button>
        {/each}
      </div>
    </div>

    <!-- Regional Folders Navigation View: If multiple folders exist and none is selected, show folders cards -->
    {#if availableFolders.length > 1 && selectedFolderName === null}
      <div class="space-y-3 mt-1">
        <div class="flex items-center justify-between">
          <div>
            <h2
              class="text-base font-bold text-neutral-200 flex items-center gap-2"
            >
              <IconFolder class="w-5 h-5 text-amber-400" />
              {$_("saveManager_folders")}
            </h2>
            <p class="text-xs text-neutral-400">
              {$_("saveManager_selectFolder")}
            </p>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each availableFolders as folder (folder.folderName)}
            <button
              type="button"
              class="flex flex-col text-left p-4 rounded-xl border bg-neutral-900/80 border-neutral-700/80 hover:border-amber-500 hover:bg-neutral-800/80 transition-all cursor-pointer group shadow-md"
              onclick={() => (selectedFolderName = folder.folderName)}
            >
              <div class="flex items-center justify-between mb-3">
                <div
                  class="p-2.5 rounded-lg bg-neutral-800 group-hover:bg-amber-500/20 text-amber-400 transition-colors"
                >
                  <IconFolder class="w-6 h-6" />
                </div>
                {#if folder.region === "NTSC-U"}
                  <Badge color="indigo" class="text-xs">NTSC-U (Americas)</Badge
                  >
                {:else if folder.region === "PAL"}
                  <Badge color="purple" class="text-xs">PAL (Europe)</Badge>
                {:else if folder.region === "NTSC-J"}
                  <Badge color="red" class="text-xs">NTSC-J (Japan)</Badge>
                {:else if folder.folderName === "default"}
                  <Badge color="gray" class="text-xs">Standard</Badge>
                {:else}
                  <Badge color="gray" class="text-xs">Custom Region</Badge>
                {/if}
              </div>

              <div class="flex-1">
                <h3
                  class="font-bold text-base text-neutral-100 group-hover:text-amber-400 transition-colors"
                >
                  {folder.displayName}
                </h3>
                <p class="text-xs font-mono text-neutral-400 mt-1 truncate">
                  {folder.folderName}
                </p>
              </div>

              <div
                class="flex items-center justify-between pt-3 mt-3 border-t border-neutral-800 text-xs text-neutral-400"
              >
                <span>
                  {folder.saves.length === 0
                    ? "0 saves"
                    : folder.saves.length === 1
                      ? "1 save"
                      : `${folder.saves.length} saves`}
                </span>
                <span
                  class="text-amber-400 font-semibold group-hover:translate-x-0.5 transition-transform"
                >
                  View Slots &rarr;
                </span>
              </div>
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <!-- Folder Header / Switcher when inside a folder -->
      {#if availableFolders.length > 1}
        <div
          class="flex flex-wrap items-center justify-between gap-3 bg-neutral-900/70 p-3 rounded-lg border border-neutral-800"
        >
          <div class="flex items-center gap-2">
            <Button
              id="btn-back-folders"
              size="xs"
              outline
              class="border-neutral-600 text-neutral-300 hover:bg-neutral-800"
              onclick={() => (selectedFolderName = null)}
            >
              <IconArrowLeft class="w-3.5 h-3.5 mr-1" />
              {$_("saveManager_allFolders")}
            </Button>
            <Tooltip
              triggeredBy="#btn-back-folders"
              placement="bottom"
              type="dark"
            >
              View all save folders and regions
            </Tooltip>

            <div class="h-4 border-l border-neutral-700 mx-1"></div>

            <IconFolder class="w-4 h-4 text-amber-400" />
            <span class="font-bold text-sm text-neutral-200">
              {activeFolder?.displayName ?? selectedFolderName}
            </span>
            {#if activeFolder?.region}
              <Badge color="indigo" class="text-xs">{activeFolder.region}</Badge
              >
            {/if}
          </div>

          <!-- Quick Folder Switcher Tabs -->
          <div class="flex items-center gap-1.5">
            {#each availableFolders as f (f.folderName)}
              <button
                type="button"
                class="px-2.5 py-1 rounded text-xs font-medium transition-all {selectedFolderName ===
                f.folderName
                  ? 'bg-amber-500 text-black font-semibold'
                  : 'bg-neutral-800 text-neutral-400 hover:bg-neutral-700 hover:text-neutral-200'}"
                onclick={() => (selectedFolderName = f.folderName)}
              >
                {f.region ?? f.folderName}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Save Slots Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-1">
        {#each slotList as item (item.id)}
          <div
            class="flex flex-col justify-between p-4 rounded-xl border transition-all {item.save
              ? 'bg-neutral-900/80 border-neutral-700/80 shadow-md hover:border-neutral-500'
              : 'bg-neutral-900/30 border-dashed border-neutral-800 opacity-60'}"
          >
            <div>
              <!-- Slot Header -->
              <div class="flex items-center justify-between mb-2">
                <span class="font-bold text-sm text-neutral-300">
                  {item.slotNumber >= 0
                    ? `${$_("saveManager_slot")} ${item.slotNumber + 1}`
                    : item.save?.baseName}
                </span>
                {#if item.save}
                  <Badge color="green" class="text-xs">Active</Badge>
                {:else}
                  <Badge
                    color="gray"
                    class="text-xs bg-neutral-800 text-neutral-400"
                  >
                    {$_("saveManager_emptySlot")}
                  </Badge>
                {/if}
              </div>

              <!-- Save Details -->
              {#if item.save}
                <div class="space-y-1.5 text-xs text-neutral-400 my-3">
                  {#if item.save.milestoneName}
                    <div class="flex items-center justify-between">
                      <span>{$_("saveManager_milestone")}:</span>
                      <span class="font-semibold text-amber-400 uppercase">
                        {item.save.milestoneName}
                      </span>
                    </div>
                  {/if}
                  <div class="flex items-center justify-between">
                    <span>Size:</span>
                    <span class="text-neutral-300"
                      >{formatBytes(item.save.sizeBytes)}</span
                    >
                  </div>
                  <div class="flex items-center justify-between">
                    <span>{$_("saveManager_lastModified")}:</span>
                    <span class="text-neutral-300"
                      >{formatDate(item.save.modifiedTimestamp)}</span
                    >
                  </div>
                  <div
                    class="text-[11px] text-neutral-500 truncate pt-1 font-mono"
                  >
                    {item.save.fileName}
                  </div>
                </div>
              {:else}
                <div class="py-8 text-center text-xs text-neutral-500 italic">
                  {$_("saveManager_noSaves")}
                </div>
              {/if}
            </div>

            <!-- Actions with Tooltips -->
            {#if item.save}
              <div
                class="pt-3 border-t border-neutral-800/80 flex items-center justify-between gap-1"
              >
                <div>
                  <Button
                    id={`btn-copy-${item.id}`}
                    size="xs"
                    class="bg-amber-600 hover:bg-amber-700 text-white font-medium flex-1 text-xs"
                    onclick={() => openTransferModal(item.save!)}
                  >
                    <IconContentCopy class="w-3.5 h-3.5 mr-1" />
                    {$_("saveManager_copyTo")}
                  </Button>
                  <Tooltip
                    triggeredBy={`#btn-copy-${item.id}`}
                    placement="top"
                    type="dark"
                  >
                    {$_("saveManager_tooltip_copy")}
                  </Tooltip>
                </div>

                <div class="flex items-center gap-1">
                  <div>
                    <Button
                      id={`btn-backup-${item.id}`}
                      size="xs"
                      outline
                      class="border-neutral-700 text-neutral-300 hover:bg-neutral-800 p-1.5"
                      onclick={() => handleBackup(item.save!)}
                      aria-label={$_("saveManager_backup")}
                    >
                      <IconBackup class="w-4 h-4 text-blue-400" />
                    </Button>
                    <Tooltip
                      triggeredBy={`#btn-backup-${item.id}`}
                      placement="top"
                      type="dark"
                    >
                      {$_("saveManager_tooltip_backup")}
                    </Tooltip>
                  </div>

                  <div>
                    <Button
                      id={`btn-delete-${item.id}`}
                      size="xs"
                      outline
                      class="border-neutral-700 text-neutral-300 hover:bg-red-950/40 hover:border-red-600 p-1.5"
                      onclick={() => openDeleteModal(item.save!)}
                      aria-label={$_("saveManager_delete")}
                    >
                      <IconDelete class="w-4 h-4 text-red-400" />
                    </Button>
                    <Tooltip
                      triggeredBy={`#btn-delete-${item.id}`}
                      placement="top"
                      type="dark"
                    >
                      {$_("saveManager_tooltip_delete")}
                    </Tooltip>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<!-- Transfer / Copy Modal -->
<Modal
  bind:open={showTransferModal}
  size="md"
  autoclose={false}
  class="bg-[#242424] text-white border border-neutral-700"
>
  <div class="p-4 space-y-4">
    <h3 class="text-lg font-bold text-white">
      {isMoveOperation ? $_("saveManager_moveTo") : $_("saveManager_copyTo")}
    </h3>

    <div>
      <label
        for="target-install"
        class="block text-xs font-semibold text-neutral-300 mb-1"
      >
        {$_("saveManager_targetInstall")}
      </label>
      <select
        id="target-install"
        bind:value={targetInstallId}
        class="w-full bg-neutral-800 border border-neutral-700 rounded p-2 text-sm text-white focus:ring-amber-500 focus:border-amber-500"
      >
        {#each installs.filter((i) => i.id !== selectedInstallId) as inst (inst.id)}
          <option value={inst.id}>
            {inst.name}
            {inst.isVanilla
              ? `(${String($_("saveManager_vanillaBadge"))})`
              : ""}
          </option>
        {/each}
      </select>
    </div>

    <!-- Destination Folder Selector if target has folders -->
    {#if targetInstall && targetInstall.folders.length > 1}
      <div>
        <label
          for="target-folder"
          class="block text-xs font-semibold text-neutral-300 mb-1"
        >
          {$_("saveManager_folders")}
        </label>
        <select
          id="target-folder"
          bind:value={targetFolderName}
          class="w-full bg-neutral-800 border border-neutral-700 rounded p-2 text-sm text-white focus:ring-amber-500 focus:border-amber-500"
        >
          {#each targetInstall.folders as f (f.folderName)}
            <option value={f.folderName}>
              {f.displayName}
            </option>
          {/each}
        </select>
      </div>
    {/if}

    <!-- Target Slot Selector -->
    <div>
      <label
        for="target-slot"
        class="block text-xs font-semibold text-neutral-300 mb-1"
      >
        {$_("saveManager_targetSlot")}
      </label>
      <select
        id="target-slot"
        bind:value={targetSlotNumber}
        class="w-full bg-neutral-800 border border-neutral-700 rounded p-2 text-sm text-white focus:ring-amber-500 focus:border-amber-500"
      >
        {#each currentGame === "jak1" ? [0, 1, 2, 3] : [0, 1, 2, 3, 4, 5, 6, 7] as slotIdx}
          <option value={slotIdx}>
            Slot {slotIdx + 1}
            {#if targetInstall?.saves.some((s) => s.slotNumber === slotIdx && (targetFolderName === "default" || s.folderName === targetFolderName))}
              (Occupied - Will overwrite)
            {/if}
          </option>
        {/each}
      </select>
    </div>

    {#if targetSlotOccupied}
      <p
        class="text-xs text-amber-400 font-medium bg-amber-950/30 p-2 rounded border border-amber-700/50"
      >
        {$_("saveManager_overwriteConfirm")}
      </p>
    {/if}

    <!-- Move Toggle -->
    <div class="flex items-center gap-2 pt-1">
      <input
        type="checkbox"
        id="move-toggle"
        bind:checked={isMoveOperation}
        class="rounded bg-neutral-800 border-neutral-700 text-amber-500 focus:ring-0"
      />
      <label for="move-toggle" class="text-xs text-neutral-300 cursor-pointer">
        {$_("saveManager_moveToggleLabel")}
      </label>
    </div>

    <!-- Modal Action Buttons -->
    <div class="flex justify-end gap-2 pt-3 border-t border-neutral-700">
      <Button
        size="sm"
        class="bg-amber-500 hover:bg-amber-600 text-black font-semibold"
        onclick={() => (showTransferModal = false)}
      >
        {$_("saveManager_cancel")}
      </Button>
      <Button
        size="sm"
        class="bg-amber-500 hover:bg-amber-600 text-black font-semibold"
        disabled={performingAction || !targetInstallId}
        onclick={executeTransfer}
      >
        {#if performingAction}
          <Spinner size="4" class="mr-2" />
        {/if}
        {$_("saveManager_confirm")}
      </Button>
    </div>
  </div>
</Modal>

<!-- Delete Confirmation Modal with Safety Warning -->
<Modal
  bind:open={showDeleteModal}
  size="sm"
  autoclose={false}
  class="bg-[#242424] text-white border border-neutral-700"
>
  <div class="p-4 space-y-4">
    <div class="flex items-center gap-3 text-red-400">
      <div
        class="p-2 rounded-full bg-red-950/60 border border-red-800/60 shrink-0"
      >
        <IconDelete class="w-6 h-6" />
      </div>
      <div>
        <h3 class="text-lg font-bold text-white">
          {$_("saveManager_delete")}
        </h3>
        <p class="text-xs text-neutral-400">
          Slot {saveToDelete?.slotNumber !== null &&
          saveToDelete?.slotNumber !== undefined &&
          saveToDelete.slotNumber >= 0
            ? saveToDelete.slotNumber + 1
            : "File"}
        </p>
      </div>
    </div>

    <p class="text-sm text-neutral-300">
      {$_("saveManager_deleteConfirm")}
    </p>

    {#if saveToDelete}
      <div
        class="bg-neutral-900/90 border border-neutral-800 p-3 rounded text-xs space-y-1.5"
      >
        <div class="flex justify-between items-center">
          <span class="text-neutral-400">File:</span>
          <span class="font-mono text-neutral-200">{saveToDelete.baseName}</span
          >
        </div>
        {#if saveToDelete.folderName && saveToDelete.folderName !== "default"}
          <div class="flex justify-between items-center">
            <span class="text-neutral-400">Folder / Region:</span>
            <span class="font-mono text-amber-300"
              >{saveToDelete.folderName}</span
            >
          </div>
        {/if}
        {#if saveToDelete.milestoneName}
          <div class="flex justify-between items-center">
            <span class="text-neutral-400">Milestone:</span>
            <span class="text-amber-400 font-semibold"
              >{saveToDelete.milestoneName}</span
            >
          </div>
        {/if}
      </div>
    {/if}

    <Alert
      color="red"
      class="bg-red-950/40 border border-red-700/60 text-red-200 text-xs p-2.5 rounded"
    >
      {$_("saveManager_deletePermanentWarning")}
    </Alert>

    <div class="flex justify-end gap-2 pt-3 border-t border-neutral-700">
      <Button
        size="sm"
        class="bg-amber-500 hover:bg-amber-600 text-black font-semibold"
        onclick={() => {
          showDeleteModal = false;
          saveToDelete = null;
        }}
      >
        {$_("saveManager_cancel")}
      </Button>
      <Button
        color="red"
        size="sm"
        disabled={performingAction}
        onclick={executeDelete}
      >
        {#if performingAction}
          <Spinner size="4" class="mr-2" />
        {/if}
        {$_("saveManager_delete")}
      </Button>
    </div>
  </div>
</Modal>
