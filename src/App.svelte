<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Router } from "sv-router";
  import { navigate, route } from "./router.ts";
  import { asJobType } from "$lib/job/jobs";
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
  let playEditedLevelListener: any = undefined;

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

    // the level editor's Play button: the level is installed and started as one of our jobs
    if (!isLevelEditor) {
      playEditedLevelListener = await listen(
        "level_edited:play",
        (event: { payload: { levelDir: string; levelName: string } }) => {
          navigate("/job/:job_type", {
            params: { job_type: asJobType("playEditedLevel") },
            search: {
              levelDir: event.payload.levelDir,
              levelName: event.payload.levelName,
              returnTo: route.pathname,
            },
          });
        },
      );
    }
  });

  onDestroy(() => {
    if (toastListener !== undefined) {
      toastListener();
    }
    if (playEditedLevelListener !== undefined) {
      playEditedLevelListener();
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
  <LevelEditor locale={config?.locale} />
{:else}
  <Router />
{/if}
