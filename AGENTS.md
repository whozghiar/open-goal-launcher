## Summary

- [1. Overview & Architecture](#1-overview--architecture)
- [2. Local Environment & Windows Tooling](#2-local-environment--windows-tooling)
- [3. AI Assistant Rules & Behavioral Protocols](#3-ai-assistant-rules--behavioral-protocols)
- [4. Modding & Runtime Pipeline](#4-modding--runtime-pipeline)
- [5. Documentation Standard](#5-documentation-standard)

---

## 1. Overview & Architecture

The **OpenGOAL Launcher** is a cross-platform desktop application designed to install, manage, update, and run OpenGOAL releases, community mods, and custom texture packs for the _Jak and Daxter_ franchise.

### Technology Stack

- **Desktop Runtime**: [Tauri v2](https://tauri.app/) (Rust backend handling native OS interactions, process management, and IPC).
- **Frontend Framework**: [Svelte 5](https://svelte.dev/) (leveraging modern runes: `$state`, `$derived`, `$props`, `$effect`).
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) & [Flowbite Svelte](https://flowbite-svelte.com/).
- **Bundler & Tooling**: [Vite](https://vitejs.dev/) with TypeScript.
- **Backend Architecture (`src-tauri/`)**:
  - `src-tauri/src/main.rs`: Application entry point, plugin registrations, and Tauri command invocation routing.
  - `src-tauri/src/config.rs`: `LauncherConfig` data models, disk persistence (`settings.json`), version migrations, and directory path resolution.
  - `src-tauri/src/cache.rs`: Metadata cache for available mods, remote schemas, and download tracking.
  - `src-tauri/src/commands/`: Domain-specific command handlers:
    - `binaries.rs`: Tooling compilation, ISO extraction, and vanilla game launch (`gk`).
    - `features/mods.rs`: Mod installation, archive extraction, CLI argument building, and mod launching.
    - `features/texture_packs.rs`: Texture pack installation and management.
    - `features/level_editor.rs`: Installs, removes and starts Jak Level Editor, a separate program downloaded per version from its GitHub releases, like the tooling.
    - `features/animation_studio.rs`: The same for Jak Animation Studio, which tries models in the games installed with the active tooling only (`--game-install`, `--tooling-version`).
    - `game.rs`: Active game detection and save milestone tracking.
    - `saves.rs`: The Saves Manager (listing, copying, moving, backing up and deleting saves).
    - `support.rs`: Support package generation (logs, config, hardware diagnostics).

---

## 2. Local Environment & Windows Tooling

On this Windows development environment, build tools and runtimes are located at the following exact paths:

| Tool                       | Path / Command                                                                    | Notes                                                                                             |
| :------------------------- | :-------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------ |
| **Rust Toolchain**         | `cargo` / `%USERPROFILE%\.cargo\bin\cargo.exe`                                    | Used for compiling and running tests in `src-tauri`.                                              |
| **Node.js Package Runner** | `npx yarn`                                                                        | Runs workspace scripts without requiring a global Yarn install.                                   |
| **Frontend Dev Server**    | `npx yarn dev`                                                                    | Starts Vite for UI browser testing.                                                               |
| **Desktop Dev Server**     | `npx yarn tauri dev`                                                              | Launches full Tauri application with live reload (Rust + Svelte).                                 |
| **Typecheck**              | `npx yarn typecheck`                                                              | Runs `svelte-check` across Svelte 5 and TypeScript files.                                         |
| **Code Formatting**        | `npx yarn check-format` / `npx yarn format`                                       | Prettier with Svelte plugin.                                                                      |
| **Rust Tests**             | `cd src-tauri && cargo test`                                                      | Runs Rust backend tests and binding export tests.                                                 |
| **Agent Skills**           | `git submodule update --init --remote .agents/skills` then `npx yarn skills:link` | Fetches the `opengoal-modding-kb` submodule (Remotion included), links it into `.claude/skills/`. |

---

## 3. AI Assistant Rules & Behavioral Protocols

When assisting on this repository, the AI must strictly adhere to the following mandatory constraints:

1. **Explicit User Notification on File Changes**:
   The AI must **never** create, modify, or delete any file without explicitly informing the user beforehand or clearly describing the modification in its response. Unreported file alterations are strictly prohibited.
2. **Strict Adherence to Initial Project Philosophy**:
   The AI must scrupulously respect the project's original philosophy, architectural separation, semantics, and syntax to minimize regressions. Never introduce unnecessary changes to unrelated files, and refactor or delete any superfluous code.
3. **No Superfluous or AI Boilerplate Comments**:
   Never add redundant, obvious, or boilerplate AI-generated comments (such as restating function names, explaining trivial lines, or typical filler like `// This function handles...`). Code must remain clean, idiomatic, and self-documenting. Comments are reserved strictly for non-obvious domain logic, upstream quirks, or critical workarounds.
4. **Strict Prohibition on Editing Generated Files**:
   Files that explicitly state not to edit them manually (such as `// This file was generated by [ts-rs]... Do not edit this file manually`) must **never** be manually edited. Updates must be performed solely through the official mechanism/tooling designated for those files.
5. **Short, English-Only Commit Messages**:
   Write each commit message in English, as one short line that says what changed in plain words, without repository-specific or obscure jargon.
6. **Mandatory AI-Assisted Suffix**:
   All commits produced with AI assistance must end with the suffix `(AI-assisted)`.
   - _Example_: `feat: allow sharing vanilla saves with mods (AI-assisted)`
7. **Mandatory Feature Documentation (`docs/features_ai/`)**:
   For every new feature implemented with AI assistance, create a dedicated markdown file named `features_[feature_name].md` inside `docs/features_ai/`, written in English and following Section 5.
8. **English-Only for i18n Translations (Crowdin-Managed)**:
   When adding or modifying user-facing text strings, only edit the primary English file (`src/assets/translations/en-US.json`). **Never** translate or edit the remaining 30+ language files manually. Crowdin manages all community translations; manual additions pollute pull requests and cause merge conflicts.
9. **Mod Metadata & Schema Collision Prevention**:
   Never introduce launcher config or state field names that conflict or overlap with mod metadata schema properties (such as introducing `shareVanillaSaves` when mod metadata defines or plans a property of the same name). Always inspect `schemas/mod-source/` to avoid naming ambiguity and collisions.
10. **Strict Prohibition on Commits without Explicit User Authorization**:
    The AI must **never** perform a `git commit` unless explicitly instructed or authorized by the user in the prompt. Code modifications must remain in the working tree for user review until a commit is explicitly requested.
11. **Strict Prohibition on Pushing without Explicit User Permission**:
    The AI is **strictly forbidden** from executing any `git push` command without explicit permission from the user.

---

## 4. Modding & Runtime Pipeline

1. **Mod Sources**: JSON feeds (defined in `schemas/mod-source/v1/`) declaring mods, versions, download links, and supported games (`jak1`, `jak2`, `jak3`, `jakx`).
2. **Directory Layout**:
   - Installed mods: `<install_dir>/features/<game>/mods/<source>/<mod_name>/`.
   - Mod settings & isolated saves: `<install_dir>/features/<game>/mods/<source>/_settings/<mod_name>/`.
3. **Execution Engine (`gk`)**:
   - Vanilla game: `gk` saves data to `%APPDATA%/OpenGOAL/<game>/saves` (Windows) or `~/.config/OpenGOAL/<game>/saves` (Linux).
   - Mods: launched with `--config-path <mod_settings_dir>`, which isolates both settings and save directories.

---

## 5. Documentation Standard

This standard applies to every `.md` file written for this repository: `README.md`, `AGENTS.md` and everything under `docs/`. Files copied from elsewhere stay as they are: `original_project_readme.md`, `schemas/README.md` and the submodule `.agents/skills/`.

### Writing rules

- **English only**: each document exists once, in English. Write no translated section, bilingual title or language switcher.
- **For beginners**: write for a reader who knows neither the code nor its jargon. Say what a thing is for before how it works, define a term the first time it appears, and lead with a concrete example (a click path, a folder, a command) rather than an abstract description.
- **Short and direct**: active voice, short sentences, one topic per section. Use a table for a mapping (an option and its effect, a field and its value) and a numbered list for steps. Skip filler phrases and closing summaries.
- **Plain formatting**: headings, lists and tables carry no emoji or decorative icon. GitHub alerts (`> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]`, `> [!CAUTION]`) are the exception, for something the reader must not skip.
- **Relative links**: link to files of the repository with relative paths (e.g. `../../src/...`), so the links work on GitHub.
- **Diagrams**: use a Mermaid `flowchart TD` with quoted labels.
- **Checked facts**: check every path, command, setting and name against the repository before writing it down.

### Feature documents (`docs/features_ai/`)

Every feature built with AI assistance gets `docs/features_ai/features_[feature_name].md`, with its screenshots in `docs/img/[feature_name]/`. [features_save_data_manager.md](docs/features_ai/features_save_data_manager.md) is a complete example.

```markdown
# <Feature name>

<One or two sentences: what the feature is and what the user can do with it.>

## Contents

- [1. What this feature brings](#1-what-this-feature-brings)
- [2. How the feature works](#2-how-the-feature-works)
- [3. How it integrates into the architecture](#3-how-it-integrates-into-the-architecture)
- [4. New files created](#4-new-files-created)
- [5. Overview of changes from the original project](#5-overview-of-changes-from-the-original-project)

## 1. What this feature brings

## 2. How the feature works

## 3. How it integrates into the architecture

## 4. New files created

## 5. Overview of changes from the original project
```

| Section                                          | What it covers                                                                                                                                                      |
| :----------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1. What this feature brings                      | The problem before the feature, then what the user gains. What the feature does not do, when a reader could expect it.                                              |
| 2. How the feature works                         | A walkthrough from the user's side, in the order they meet it: where to click, what each screen shows, what happens after each button. Screenshots where they help. |
| 3. How it integrates into the architecture       | The path from the interface (Svelte) through the Tauri commands (Rust) to the files and the game, with links to the main files.                                     |
| 4. New files created                             | Each new file, linked, with one line on its role.                                                                                                                   |
| 5. Overview of changes from the original project | What changed in existing files, grouped by area, without minor details.                                                                                             |
