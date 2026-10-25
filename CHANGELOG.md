# Changelog

This page lists what each version of this launcher changes, newest first. Each entry starts with what you will notice, then, under **Under the hood**, what changed for those who build the launcher. The versions of the original OpenGOAL launcher are listed in [its own releases](https://github.com/open-goal/launcher/releases).

## 1.1.2 (2026-10-25)

Games work again on macOS: the launcher is back on the official OpenGOAL tools.

- **The games use the official OpenGOAL tooling again**, from [open-goal/jak-project](https://github.com/open-goal/jak-project), as in the original launcher. Version 1.1 had switched to a fork that has no macOS build, so a Mac could not install the tooling at all. If your active tooling is a build of that fork (`v0.3.9-whozghiar.1`), the header now offers the official version: download it in **Settings > Versions**. The fork build stays in the list until you remove it.
- **Older official versions can be downloaded again.** Since 1.1.0, the launcher refused any file that GitHub gives no checksum for, which is the case of every official release published before mid-2025. It now checks a file whenever GitHub has its checksum, and downloads the others as the launcher always did.
- **The Level Editor button shows on macOS**, with a message: the editor has no macOS version yet, and work is in progress.

The Level Editor itself does not change: it keeps updating on its own, from **Settings > Level Editor**. The levels it exports are still built with the fork [whozghiar/jak-project](https://github.com/whozghiar/jak-project), in a jak-project of your own, as the export's README explains.

**Under the hood**: `TOOLING_REPOSITORY` in `src/lib/utils/github.ts` is `open-goal/jak-project` again. `download_release_asset` in `src-tauri/src/util/network.rs` accepts an asset GitHub has no digest for, and still refuses one that differs from its digest.

## 1.1.1 (2026-10-23)

The Level Editor becomes a separate program, which updates without a new launcher.

- **Install the editor from Settings > Level Editor.** The launcher downloads the version you choose (about 10 MB) and makes it active. Then click **Level Editor** in the sidebar to open it.
- **A download icon in the header** tells you when a newer editor is published, as it does for the launcher and the tooling. Nothing installs without a click.
- **Your editor keeps its library and preferences**, and opens in the launcher's language.

**Under the hood**: the editor is no longer compiled into the launcher. The `level-editor/` submodule and its Tauri plugin are gone: the launcher downloads the portable build of [whozghiar/jak-level-editor](https://github.com/whozghiar/jak-level-editor) into `<install>/versions/level-editor/<version>/` and starts it. Guide: [docs/features_ai/features_level_editor.md](docs/features_ai/features_level_editor.md).

## 1.1.0 (2026-10-23)

The Level Editor arrives in the launcher.

- **A Level Editor button** in the sidebar opens Jak Level Editor, to edit the levels of Jak 3 and export them as custom levels. It reads the disc the launcher already extracted. Not available on macOS.
- **OpenGOAL tools from a fork**: the launcher installed and updated OpenGOAL from [whozghiar/jak-project](https://github.com/whozghiar/jak-project). Version 1.1.2 goes back to the official tools.
- **Checked downloads**: the tooling's archives are checked against the checksum GitHub publishes for them.

**Under the hood**: the editor was a git submodule compiled into the launcher as a Tauri plugin, replaced in 1.1.1.

## 1.0.2 (2026-10-10)

- **Fix**: texture packs apply to a mod that imports custom levels.

## 1.0.1 (2026-10-10)

- **Fix**: the automatic update reads the releases of this launcher.

## 1.0.0 (2026-10-10)

The first version of this fork of the launcher.

- **Saves Manager**: one screen lists the saves of the base game and of every installed mod, and lets you copy, move, back up or delete them. Open it from the **Advanced** menu of a game or a mod. Guide: [docs/features_ai/features_save_data_manager.md](docs/features_ai/features_save_data_manager.md).
- **Mod Texture Packs**: texture packs work for mods, each mod with its own packs, kept apart from the base game. Open them from the **Texture Packs** button of a mod. Guide: [docs/features_ai/feature_modTextures.md](docs/features_ai/feature_modTextures.md).
