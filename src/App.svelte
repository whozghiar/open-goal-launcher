<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Router } from "sv-router";
  import "./router.ts";
  import { isInDebugMode } from "$lib/utils/common";
  import { isMinimumVCCRuntimeInstalled, setLocale } from "$lib/rpc/config";
  import { listen } from "@tauri-apps/api/event";
  import { toastStore, type ToastLevel } from "$lib/stores/ToastStore";
  import { systemInfoState } from "./state/SystemInfoState.svelte.ts";
  import { config } from "./state/config.svelte.ts";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import LevelEditor from "$level-editor/LevelEditor.svelte";
  import { LEVEL_EDITOR_WINDOW } from "$lib/level-editor/window";

  const isLevelEditor = getCurrentWebviewWindow().label === LEVEL_EDITOR_WINDOW;

  let revokeSpecificActions = false;
  let toastListener: any = undefined;

  // Events
  onMount(async () => {
    // Set locale from settings
    const locale = config?.locale;
    if (locale) {
      setLocale(locale);
    }

    systemInfoState.isMinVCCRuntimeInstalled =
      await isMinimumVCCRuntimeInstalled();

    toastListener = await listen(
      "toast_msg",
      (event: { payload: { toast: string; level: ToastLevel } }) => {
        toastStore.makeToast(event.payload.toast, event.payload.level);
      },
    );
  });

  onDestroy(() => {
    if (toastListener !== undefined) {
      toastListener();
    }
  });

  if (!isInDebugMode()) {
    revokeSpecificActions = true;
    // Disable Right Click
    document.addEventListener("contextmenu", (event) => {
      if (revokeSpecificActions) {
        event.preventDefault();
      }
    });
    // Disable Refreshing (F5 / Ctrl+R)
    document.addEventListener("keydown", (e) => {
      if (e.code == "F5") {
        if (revokeSpecificActions) {
          e.preventDefault();
        }
      }
      if (e.code == "KeyR" && e.ctrlKey) {
        if (revokeSpecificActions) {
          e.preventDefault();
        }
      }
      // super secret keybind to reverse the above so we can debug a release version
      // Shift+Ctrl F12
      if (e.code == "F12" && e.ctrlKey && e.shiftKey) {
        revokeSpecificActions = false;
      }
    });
  }
</script>

{#if isLevelEditor}
  <LevelEditor embedded locale={config?.locale} />
{:else}
  <Router />
{/if}
