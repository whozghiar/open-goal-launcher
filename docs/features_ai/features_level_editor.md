# Level Editor

The **Level Editor** button in the sidebar starts Jak Level Editor, a level editor for the Jak games. The launcher installs and updates the editor the way it does the OpenGOAL tools: each version is downloaded from a GitHub release, and you choose which one is active. This page explains what the editor offers, how the launcher installs, updates and starts it, and what changed in the launcher to make that possible.

## Contents

- [1. What this feature brings](#1-what-this-feature-brings)
- [2. How the feature works](#2-how-the-feature-works)
- [3. How it integrates into the architecture](#3-how-it-integrates-into-the-architecture)
- [4. New files created](#4-new-files-created)
- [5. Overview of changes from the original project](#5-overview-of-changes-from-the-original-project)

## 1. What this feature brings

Jak Level Editor is a separate project, [jak-level-editor](https://github.com/whozghiar/jak-level-editor), written in Rust and Svelte like the launcher. With it you can:

- open any level of the game, read from your own disc;
- move, copy and delete its pieces, and place objects, prefabs and whole buildings;
- paint terrain;
- see the level lit as the game lights it, at any hour of the day;
- generate the paths that creatures walk on;
- export the result as a custom level that OpenGOAL builds and plays.

The editor works on Jak 3 today. Its **Game** menu also lists Jak and Daxter and Jak II, marked _Work in progress_. It does not play levels: it exports them as files to copy into a jak-project (the OpenGOAL source code), with a README and the guide's tutorial _Add your level to the game by hand_.

What the launcher adds:

- **Updates without a launcher release.** The editor is a program of its own, downloaded from the editor's releases. A new editor version reaches users as soon as it is published.
- **An alert when a version is out.** A download icon appears next to the editor's version in the header, as it does for the launcher and the tooling.
- **Versions you control.** **Settings > Level Editor** lists the versions: download one, make it active, go back to an older one, or delete it.
- **What the launcher already knows.** The editor gets the launcher's language and starts its extraction from the discs the launcher already extracted. It keeps the same preferences whether you open it from the launcher or on its own.

The games keep the official OpenGOAL tooling, from [open-goal/jak-project](https://github.com/open-goal/jak-project). The levels the editor exports are built in a jak-project with the fork [whozghiar/jak-project](https://github.com/whozghiar/jak-project), which carries the level builder fixes they need; the launcher does not install that fork.

> [!NOTE]
> The editor has a build for Windows, Linux and macOS (one for both Apple Silicon and Intel Macs) since its version v1.1.0. On macOS, **Settings > Level Editor** lists only the versions that have one: v1.1.0 and later.

## 2. How the feature works

### Installing the editor

1. Click **Level Editor** (pencil-and-ruler icon) in the sidebar, above Settings.

   ![Sidebar of the launcher with the Level Editor button and its tooltip](../img/level_editor/capture_1.png)

2. No version is installed yet, so the launcher opens **Settings > Level Editor**, the list of the editor's releases.
3. Click **Download** on the latest version. The launcher downloads the editor's portable build for your system (8 MB on Windows, 15 MB on macOS, with its translations and data), checks it, unpacks it and makes it the active version.
4. Click **Level Editor** again. The editor opens in its own window.

### Updating the editor

When the launcher starts, it compares the editor's latest release with the active version. If the latest one is newer and not downloaded yet, a pulsing download icon appears next to **Level Editor** in the header. Click it to open **Settings > Level Editor**, then download the new version: it becomes the active one.

Nothing is installed without a click. The launcher handles its four components the same way:

| Component        | Where the alert shows                       | What installs the update                                                     |
| :--------------- | :------------------------------------------ | :--------------------------------------------------------------------------- |
| Launcher         | Icon next to the launcher's version         | The update page: one click downloads, installs and restarts the launcher.    |
| Tooling          | Icon next to the tooling's version          | **Settings > Versions**: download the version, it becomes active.            |
| Level Editor     | Icon next to the Level Editor's version     | **Settings > Level Editor**: download the version, it becomes active.        |
| Animation Studio | Icon next to the Animation Studio's version | **Settings > Animation Studio** ([its guide](features_animation_studio.md)). |

All four checks run once, when the launcher starts. Older versions stay on disk until you delete them, so you can make one active again from the list.

### What the launcher gives the editor

The launcher starts the editor with these settings:

| What            | How                                                                                 | Effect                                                                                                                          |
| :-------------- | :---------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------ |
| Language        | `--lang fr`, from the launcher's language                                           | The editor's texts in that language, when the editor has it (English, French).                                                  |
| Preferences     | Nothing: the editor keeps them in its own folder, `%APPDATA%/OpenGOAL/level-editor` | The same library, language and recent projects as when you open the editor on its own.                                          |
| Default library | `--default-library=<install>/jak-level-editor`                                      | The folder the first-run assistant offers for the library.                                                                      |
| Games           | `--game-install=jak3=<install>`, one per game installed with the active version     | **File > Extract game assets** reads the game's levels with its own extractor; **Level > Try in the game** plays a level in it. |

Earlier, the launcher gave the editor its own preferences folder, `%APPDATA%/OpenGOAL-Launcher/jak-level-editor` (the `JLE_PREFS_DIR` environment variable). The first time an editor newer than v1.1.0 starts, it moves the newest of its earlier folders into `%APPDATA%/OpenGOAL/level-editor` and deletes the others: `%APPDATA%/OpenGOAL/jak-level-editor` (the editor on its own), `%APPDATA%/OpenGOAL-Launcher/jak-level-editor` and `%APPDATA%/OpenGOAL/open-goal-level-editor` (the C++ editor). Versions up to v1.1.0 still keep theirs in `%APPDATA%/OpenGOAL/jak-level-editor`.

> [!IMPORTANT]
> `--default-library` and `--game-install` need an editor version that knows them, newer than v1.0.0. Older versions skip them whole, because they are written as `--name=value`. With such a version, the first-run assistant offers `Documents/Jak Level Editor` and the editor asks for the game's folder in **File > Settings**.

### One editor at a time

If you click **Level Editor** while the editor it started is still open, the launcher shows "The Level Editor is already open" instead of starting a second one on the same preferences.

The editor's output goes to `level-editor.log`, in the launcher's log folder.

## 3. How it integrates into the architecture

### Files on disk

| Location                                     | Content                                                                                                                                  |
| :------------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------- |
| `<install>/versions/level-editor/<version>/` | One downloaded version: `jak-level-editor(.exe)`, `lang/` and `data/` (and `ogle-extract`, the extractor of the versions before v1.1.0). |
| `settings.json`, field `levelEditorVersion`  | The active version.                                                                                                                      |
| `%APPDATA%/OpenGOAL/level-editor/`           | The editor's preferences and caches, the same whoever starts it.                                                                         |
| `<install>/jak-level-editor/`                | The default library: extracted levels, projects, exports.                                                                                |

### From the button to the editor

```mermaid
flowchart TD
  A["Sidebar: Level Editor button"] --> B["openLevelEditor() in launch.ts"]
  B -->|"No active version"| C["Settings > Level Editor"]
  B -->|"Active version"| D["Rust command launch_level_editor"]
  D -->|"Already open"| E["Toast: already open"]
  D -->|"Otherwise"| F["Starts versions/level-editor/version/jak-level-editor with the launcher's options"]
```

[launch.ts](../../src/lib/level-editor/launch.ts) holds the button's logic. The button, the header and the settings show the editor on every system.

### Versions and updates

The editor reuses the tooling's version code, with its own repository and files:

- [github.ts](../../src/lib/utils/github.ts): `LEVEL_EDITOR_REPOSITORY` is `whozghiar/jak-level-editor`. `listLevelEditorReleases` and `getLatestLevelEditorRelease` share their code with the tooling's functions and pick the asset `jak-level-editor-portable-<platform>.zip` (`portableDownloadLink`, shared with the Animation Studio), where `<platform>` is `windows-x64`, `linux-x64` or `macos-universal`.
- [Header.svelte](../../src/components/header/Header.svelte): `checkForProgramUpdate`, next to `checkForToolingUpdate`, sets `selectedLevelEditor` in `UpdateStore` ([AppStore.ts](../../src/lib/stores/AppStore.ts)); it checks the Animation Studio the same way.
- [Versions.svelte](../../src/routes/settings/Versions.svelte): one page for both. `kind="levelEditor"` picks the editor's commands, folder and texts. [Latest.svelte](../../src/routes/settings/versions/Latest.svelte) and [VersionList.svelte](../../src/routes/settings/versions/VersionList.svelte) take the active version to show through `activeKey`, from [VersionState.svelte.ts](../../src/state/VersionState.svelte.ts).

The Rust commands, in [level_editor.rs](../../src-tauri/src/commands/features/level_editor.rs) except the first:

| Command                         | What it does                                                                                                                                                                                          |
| :------------------------------ | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `list_downloaded_versions`      | The tooling's command, with the folder `level-editor`: lists the versions on disk.                                                                                                                    |
| `download_level_editor_version` | Downloads the release's portable build, checks it against GitHub's SHA-256 of it (`download_release_asset` in [network.rs](../../src-tauri/src/util/network.rs)), unpacks it into its version folder. |
| `remove_level_editor_version`   | Deletes a version folder, and clears the active version if it was that one.                                                                                                                           |
| `set_level_editor_version`      | Saves the active version in `settings.json`.                                                                                                                                                          |
| `launch_level_editor`           | Starts the active version with the options above. Answers `false` when the editor it started before is still running.                                                                                 |

### Publishing a new editor version

Run the release workflow of [jak-level-editor](https://github.com/whozghiar/jak-level-editor). Once its release is published, not left as a draft, with the `jak-level-editor-portable-<platform>.zip` files, launchers offer it at their next start. The launcher needs no release of its own.

### The games' tooling

`TOOLING_REPOSITORY` in [github.ts](../../src/lib/utils/github.ts) is `open-goal/jak-project`: the Versions page and the header's update icon follow the official releases, which have builds for Windows, Linux and macOS.

- Launcher 1.1 installed builds of the fork instead, tagged `vX.Y.Z-whozghiar.N`. The Versions page keeps listing an installed version that GitHub does not list, so it can still be chosen again or removed, and `tooling_version` in [config.rs](../../src-tauri/src/config.rs) reads such a version as X.Y.Z when the launcher checks minimum versions.
- Tooling downloads go through `download_release_asset` too ([versions.rs](../../src-tauri/src/commands/versions.rs)). GitHub has a SHA-256 for the assets uploaded since mid-2025 only: an official release older than that is downloaded unchecked, as the launcher always did.

## 4. New files created

| File                                                                                                       | Role                                                 |
| :--------------------------------------------------------------------------------------------------------- | :--------------------------------------------------- |
| [`src/lib/level-editor/launch.ts`](../../src/lib/level-editor/launch.ts)                                   | Starts the editor, or opens its versions page.       |
| [`src-tauri/src/commands/features/level_editor.rs`](../../src-tauri/src/commands/features/level_editor.rs) | Downloads, removes and starts the editor's versions. |
| [`docs/features_ai/features_level_editor.md`](features_level_editor.md)                                    | This document.                                       |
| [`docs/img/level_editor/capture_1.png`](../img/level_editor/capture_1.png)                                 | The screenshot of the sidebar button.                |

## 5. Overview of changes from the original project

- **Sidebar**: [Sidebar.svelte](../../src/components/sidebar/Sidebar.svelte) gains the **Level Editor** button.
- **Header**: [Header.svelte](../../src/components/header/Header.svelte) shows the editor's version and its update icon.
- **Settings**: [Settings.svelte](../../src/routes/Settings.svelte) gains the **Level Editor** tab. The Versions page and its two components serve both the tooling and the editor.
- **Releases**: [github.ts](../../src/lib/utils/github.ts) reads the editor from its own repository with the tooling's code.
- **Settings file**: [config.rs](../../src-tauri/src/config.rs) adds `levelEditorVersion`, and `tooling_version` so a fork build installed by launcher 1.1 meets the minimums of its X.Y.Z. The TypeScript type [LauncherConfig.ts](../../src/lib/rpc/bindings/LauncherConfig.ts) is regenerated by `ts-rs`.
- **Backend**: [main.rs](../../src-tauri/src/main.rs) registers the editor's commands. [network.rs](../../src-tauri/src/util/network.rs) adds `download_release_asset`, which checks a release asset against its SHA-256 when GitHub has one, used for the editor and the tooling.
- **Versions page**: [Versions.svelte](../../src/routes/settings/Versions.svelte) keeps listing installed versions that GitHub does not list.
- **Translations**: the editor's texts (`sidebar_levelEditor`, `header_levelEditorVersionLabel`, `settings_tabs_levelEditor`, `settings_versions_levelEditor_*`, `toasts_savedLevelEditorVersion`, `toasts_levelEditorAlreadyOpen`) were added to [en-US.json](../../src/assets/translations/en-US.json).

Earlier builds of this fork compiled the editor into the launcher, as a submodule (`level-editor/`) and a Tauri plugin. Downloading it as a separate program lets it update without a launcher release, and the launcher builds without the editor's code.
