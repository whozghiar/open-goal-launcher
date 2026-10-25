import { platform } from "@tauri-apps/plugin-os";
import { unwrapFunctionStore, format } from "svelte-i18n";
import { launchLevelEditor } from "$lib/rpc/features";
import { toastStore } from "$lib/stores/ToastStore";
import { navigate } from "/src/router";
import { config } from "/src/state/config.svelte";

const $format = unwrapFunctionStore(format);

// not on macOS for now: the editor has no build for it (its extractor is x86 only); its button
// says it is coming
export const LEVEL_EDITOR_AVAILABLE = platform() !== "macos";

// Jak Level Editor is a program of its own, downloaded per version like the tooling: without an
// active version, its page of the settings offers them
export async function openLevelEditor() {
  if (!LEVEL_EDITOR_AVAILABLE) {
    toastStore.makeToast($format("toasts_levelEditorComingToMacos"), "info");
    return;
  }
  if (!config?.levelEditorVersion) {
    navigate("/settings/:tab", { params: { tab: "level-editor" } });
    return;
  }
  if (!(await launchLevelEditor())) {
    toastStore.makeToast($format("toasts_levelEditorAlreadyOpen"), "info");
  }
}
