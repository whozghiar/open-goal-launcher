import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { platform } from "@tauri-apps/plugin-os";
import { refreshLevelEditorHost } from "$lib/rpc/features";

export const LEVEL_EDITOR_WINDOW = "level-editor";

// not on macOS for now: whozghiar/jak-project's tooling and ogle-extract have no build for it
export const LEVEL_EDITOR_AVAILABLE = platform() !== "macos";

// Jak Level Editor (level-editor/, a submodule) runs in a window of its own, the main window's
// frontend showing it instead of the launcher when the window has this label (App.svelte)
export async function openLevelEditor() {
  // what the editor gets from the launcher depends on its folders: the installation folder, the
  // games installed, ogle-extract downloaded
  await refreshLevelEditorHost();
  const existing = await WebviewWindow.getByLabel(LEVEL_EDITOR_WINDOW);
  if (existing) {
    await existing.unminimize();
    await existing.setFocus();
    return;
  }
  new WebviewWindow(LEVEL_EDITOR_WINDOW, {
    url: "/",
    title: "Jak Level Editor",
    width: 1600,
    height: 900,
    minWidth: 1100,
    minHeight: 650,
    center: true,
    decorations: false,
    maximized: true,
  });
}
