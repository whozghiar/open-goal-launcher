<script lang="ts">
  import IconArrowLeft from "~icons/mdi/arrow-left";
  import IconFolder from "~icons/mdi/folder-open";
  import IconContentCopy from "~icons/mdi/content-copy";
  import IconDelete from "~icons/mdi/delete";
  import IconBackup from "~icons/mdi/backup-restore";
  import IconAlert from "~icons/mdi/alert-circle";
  import IconRefresh from "~icons/mdi/refresh";
  import {
    Alert,
    Badge,
    Button,
    Modal,
    Select,
    Spinner,
    Tooltip,
  } from "flowbite-svelte";
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { openPath } from "@tauri-apps/plugin-opener";
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
  } from "$lib/rpc/saves";
  import type { SaveInstallInfo } from "$lib/rpc/bindings/SaveInstallInfo";
  import type { SaveSlotInfo } from "$lib/rpc/bindings/SaveSlotInfo";

  const gameParam = $derived(route.params.game_name);
  let activeGame: SupportedGame | undefined = $state(undefined);

  $effect(() => {
    const activeGameFromParam = toSupportedGame(gameParam);
    if (activeGameFromParam) {
      activeGame = activeGameFromParam;
    }
  });

  let loaded = $state(false);
  let loadingInstalls = $state(false);
  let installs: SaveInstallInfo[] = $state([]);
  let selectedInstallId: string = $state("vanilla");

  // Transfer Modal State
  let showTransferModal = $state(false);
  let transferSourceSave: SaveSlotInfo | null = $state(null);
  let targetInstallId: string = $state("");
  let targetSlotNumber: number = $state(0);
  let isMoveOperation: boolean = $state(false);
  let performingAction = $state(false);

  // Delete Confirmation State
  let showDeleteModal = $state(false);
  let saveToDelete: SaveSlotInfo | null = $state(null);

  let activeInstall = $derived(
    installs.find((i) => i.id === selectedInstallId) ?? installs[0],
  );

  let targetInstall = $derived(
    installs.find((i) => i.id === targetInstallId) ?? null,
  );

  let isTransferBlocked = $derived.by(() => {
    if (!activeInstall || !targetInstall) return false;
    const involvesVanilla = activeInstall.isVanilla || targetInstall.isVanilla;
    const hasIncompatibleMod =
      activeInstall.hasCustomSaveFormat || targetInstall.hasCustomSaveFormat;
    return involvesVanilla && hasIncompatibleMod;
  });

  let targetSlotOccupied = $derived.by(() => {
    if (!targetInstall) return false;
    return targetInstall.saves.some((s) => s.slotNumber === targetSlotNumber);
  });

  onMount(async () => {
    await refreshInstalls();
    loaded = true;
  });

  async function refreshInstalls() {
    if (!activeGame) return;
    loadingInstalls = true;
    try {
      installs = await listGameSaveInstalls(activeGame);
      if (
        !installs.some((i) => i.id === selectedInstallId) &&
        installs.length > 0
      ) {
        selectedInstallId = installs[0].id;
      }
    } catch (err) {
      toastStore.makeToast(`Failed to load saves: ${err}`, "error");
    } finally {
      loadingInstalls = false;
    }
  }

  function openTransferModal(save: SaveSlotInfo) {
    transferSourceSave = save;
    const otherInstalls = installs.filter((i) => i.id !== selectedInstallId);
    targetInstallId = otherInstalls.length > 0 ? otherInstalls[0].id : "";
    targetSlotNumber = save.slotNumber ?? 0;
    isMoveOperation = false;
    showTransferModal = true;
  }

  async function executeTransfer() {
    if (!activeGame || !transferSourceSave || !targetInstallId) return;
    if (isTransferBlocked) return;

    performingAction = true;
    try {
      if (isMoveOperation) {
        await moveSave(
          activeGame,
          selectedInstallId,
          targetInstallId,
          transferSourceSave.fileName,
          targetSlotNumber,
          true,
        );
        toastStore.makeToast($_("saveManager_moveSuccess"), "info");
      } else {
        await copySave(
          activeGame,
          selectedInstallId,
          targetInstallId,
          transferSourceSave.fileName,
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
    if (!activeGame) return;
    performingAction = true;
    try {
      const backupName = await backupSave(
        activeGame,
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
    if (!activeGame || !saveToDelete) return;
    performingAction = true;
    try {
      await deleteSave(activeGame, selectedInstallId, saveToDelete.fileName);
      toastStore.makeToast($_("saveManager_deleteSuccess"), "info");
      showDeleteModal = false;
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
      await openPath(activeInstall.saveDir);
    } catch (err) {
      toastStore.makeToast(`Unable to open folder: ${err}`, "error");
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

  // Pre-generate 4 visual slots for Jak 1
  let slotList = $derived.by(() => {
    if (!activeInstall) return [];
    const maxSlots = 4;
    const slots = [];
    for (let slotIdx = 0; slotIdx < maxSlots; slotIdx++) {
      const existing = activeInstall.saves.find(
        (s) => s.slotNumber === slotIdx,
      );
      slots.push({
        slotNumber: slotIdx,
        save: existing ?? null,
      });
    }
    // Include extra saves that did not match standard slot numbering (e.g. backup files)
    const extraSaves = activeInstall.saves.filter(
      (s) => s.slotNumber === null || s.slotNumber >= maxSlots,
    );
    for (const extra of extraSaves) {
      slots.push({
        slotNumber: extra.slotNumber ?? -1,
        save: extra,
      });
    }
    return slots;
  });
</script>

<div class="flex flex-col min-h-full flex-1 bg-[#1e1e1e] p-5 gap-4 text-white">
  {#if !loaded || !activeGame}
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
          outline
          class="border-solid rounded text-white hover:dark:text-slate-900 hover:bg-white font-semibold p-2.5"
          onclick={() => {
            if (activeGame) {
              navigate(`/:game_name/`, { params: { game_name: activeGame } });
            }
          }}
          aria-label={$_("saveManager_back")}
        >
          <IconArrowLeft class="w-5 h-5" />
        </Button>
        <div>
          <h1 class="text-xl font-bold tracking-wide">
            {$_("saveManager_title")}
          </h1>
          <p class="text-xs text-neutral-400">
            {$_("saveManager_subtitle")}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Button
          size="sm"
          outline
          class="border-neutral-600 text-neutral-200 hover:bg-neutral-800"
          onclick={refreshInstalls}
          disabled={loadingInstalls}
        >
          <IconRefresh
            class="w-4 h-4 mr-1 {loadingInstalls ? 'animate-spin' : ''}"
          />
          {$_("saveManager_refresh")}
        </Button>

        {#if activeInstall}
          <Button
            size="sm"
            class="bg-blue-600 hover:bg-blue-700 text-white font-medium"
            onclick={openCurrentSaveFolder}
          >
            <IconFolder class="w-4 h-4 mr-1.5" />
            {$_("saveManager_openFolder")}
          </Button>
        {/if}
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
            }}
          >
            <span>{inst.name}</span>
            {#if inst.isVanilla}
              <Badge color="blue" class="text-xs px-1.5 py-0.5">
                {$_("saveManager_vanillaBadge")}
              </Badge>
            {:else if inst.hasCustomSaveFormat}
              <Badge
                color="red"
                class="text-xs px-1.5 py-0.5 flex items-center gap-1"
              >
                <IconAlert class="w-3 h-3" />
                Custom
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

    <!-- Incompatible Mod High-Visibility Warning -->
    {#if activeInstall?.hasCustomSaveFormat}
      <Alert
        color="yellow"
        class="border-2 border-amber-500/80 bg-amber-950/40 text-amber-200 rounded-lg p-4 shadow-lg"
      >
        <div class="flex items-start gap-3">
          <IconAlert class="w-6 h-6 text-amber-400 shrink-0 mt-0.5" />
          <div class="flex flex-col gap-1">
            <span class="text-base font-bold text-amber-300">
              {$_("saveManager_warningIncompatibleTitle")}
            </span>
            <p class="text-sm text-amber-100/90 leading-relaxed">
              {activeInstall.warningMessage ??
                $_("saveManager_warningIncompatibleMessage")}
            </p>
          </div>
        </div>
      </Alert>
    {/if}

    <!-- Save Slots Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-2">
      {#each slotList as item (item.slotNumber + (item.save?.fileName ?? "empty"))}
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
                  : item.save?.fileName}
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

          <!-- Actions -->
          {#if item.save}
            <div
              class="pt-3 border-t border-neutral-800/80 flex items-center justify-between gap-1"
            >
              <Button
                size="xs"
                class="bg-amber-600 hover:bg-amber-700 text-white font-medium flex-1 text-xs"
                onclick={() => openTransferModal(item.save!)}
              >
                <IconContentCopy class="w-3.5 h-3.5 mr-1" />
                {$_("saveManager_copyTo")}
              </Button>
              <Button
                size="xs"
                outline
                class="border-neutral-700 text-neutral-300 hover:bg-neutral-800 p-1.5"
                onclick={() => handleBackup(item.save!)}
                aria-label={$_("saveManager_backup")}
              >
                <IconBackup class="w-4 h-4 text-blue-400" />
              </Button>
              <Button
                size="xs"
                outline
                class="border-neutral-700 text-neutral-300 hover:bg-red-950/40 hover:border-red-600 p-1.5"
                onclick={() => openDeleteModal(item.save!)}
                aria-label={$_("saveManager_delete")}
              >
                <IconDelete class="w-4 h-4 text-red-400" />
              </Button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
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
            {inst.hasCustomSaveFormat ? "(Incompatible)" : ""}
          </option>
        {/each}
      </select>
    </div>

    <!-- Mod Incompatibility Warning inside modal -->
    {#if isTransferBlocked}
      <Alert
        color="red"
        class="border border-red-600 bg-red-950/60 text-red-200 text-xs p-3 rounded"
      >
        <div class="flex items-start gap-2">
          <IconAlert class="w-5 h-5 text-red-400 shrink-0" />
          <span>{$_("saveManager_disabledTransferTooltip")}</span>
        </div>
      </Alert>
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
        {#each [0, 1, 2, 3] as slotIdx}
          <option value={slotIdx}>
            Slot {slotIdx + 1}
            {#if targetInstall?.saves.some((s) => s.slotNumber === slotIdx)}
              (Occupied - Will overwrite)
            {/if}
          </option>
        {/each}
      </select>
    </div>

    {#if targetSlotOccupied && !isTransferBlocked}
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
        color="alternative"
        size="sm"
        onclick={() => (showTransferModal = false)}
      >
        {$_("saveManager_cancel")}
      </Button>
      <Button
        size="sm"
        class="bg-amber-500 hover:bg-amber-600 text-black font-semibold"
        disabled={performingAction || isTransferBlocked || !targetInstallId}
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

<!-- Delete Confirmation Modal -->
<Modal
  bind:open={showDeleteModal}
  size="sm"
  autoclose={false}
  class="bg-[#242424] text-white border border-neutral-700"
>
  <div class="p-4 space-y-4">
    <h3 class="text-lg font-bold text-red-400 flex items-center gap-2">
      <IconDelete class="w-5 h-5" />
      {$_("saveManager_delete")}
    </h3>
    <p class="text-sm text-neutral-300">
      {$_("saveManager_deleteConfirm")}
    </p>
    {#if saveToDelete}
      <div
        class="text-xs text-neutral-400 bg-neutral-900 p-2 rounded font-mono"
      >
        {saveToDelete.fileName}
      </div>
    {/if}
    <div class="flex justify-end gap-2 pt-3 border-t border-neutral-700">
      <Button
        color="alternative"
        size="sm"
        onclick={() => (showDeleteModal = false)}
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
