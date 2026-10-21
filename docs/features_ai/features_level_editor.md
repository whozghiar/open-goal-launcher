> **Language / Langue :** [🇬🇧 English Version](#-english-version) &nbsp;•&nbsp; [🇫🇷 Version Française](#-version-française)

## Summary / Sommaire

- [🇬🇧 English Version](#-english-version)
  - [1. What This Feature Brings](#1-what-this-feature-brings)
  - [2. How the Feature Works](#2-how-the-feature-works)
  - [3. How it Integrates into the Architecture](#3-how-it-integrates-into-the-architecture)
  - [4. New Files Created](#4-new-files-created)
  - [5. Overview of Changes from the Original Project](#5-overview-of-changes-from-the-original-project)
- [🇫🇷 Version Française](#-version-française)
  - [1. Qu'est-ce qu'elle apporte](#1-quest-ce-quelle-apporte)
  - [2. Comment fonctionne la fonctionnalité](#2-comment-fonctionne-la-fonctionnalité)
  - [3. Comment elle s'intègre dans l'architecture](#3-comment-elle-sintègre-dans-larchitecture)
  - [4. Quels sont les nouveaux fichiers](#4-quels-sont-les-nouveaux-fichiers)
  - [5. Quels sont les modifications dans les grandes lignes](#5-quels-sont-les-modifications-dans-les-grandes-lignes)

---

# 🇬🇧 English Version

## 1. What This Feature Brings

A **Level Editor** button in the sidebar opens **Jak Level Editor**, a level editor for Jak 3, in a
window of its own. With it you can open any level of the game (read from your own disc), move,
copy and delete its pieces, place objects, prefabs and whole buildings, paint terrain, see the level
lit as the game lights it at any hour, generate the paths creatures walk on, and export the result
as a custom level that OpenGOAL builds and plays. An AI assistant can also drive it (MCP server).

In the launcher, the editor also **plays** what you build: its **Play** button installs the level
in a game of its own, apart from your Jak 3, its saves and your mods, and starts it there, with the
launcher's progress page. The disc the launcher already extracted is the editor's source, and the
editor's extractor is downloaded the first time it is needed.

The editor is its own project ([open-goal-level-builder](https://github.com/whozghiar/open-goal-level-builder)),
written in Rust and Svelte like the launcher. The launcher only plugs it in: it keeps no copy of
its code.

## 2. How the Feature Works

1. The user clicks **Level Editor** (pencil-and-ruler icon) in the sidebar.

   ![Sidebar of the launcher with the Level Editor button and its tooltip](../img/level_editor/capture_1.png)

2. The launcher opens a second window labelled `level-editor`, maximized, or brings it to the front
   (restored if it was minimized) when it is already open. Closing the launcher's main window leaves
   the editor open; unsaved work is never lost silently (the editor asks before closing).
3. That window loads the launcher's own interface, which sees its label and shows the editor
   instead of the launcher's pages. Like the launcher's window, it has no system frame: the editor
   draws its own title bar (the level's name, an orange dot when it is not saved, minimize,
   maximize, close). Below it come the menus (File, Edit, View, Build, Level, Help), the toolbar
   (move, rotate, scale, topology, terrain, copy, paste, delete, create a prefab, undo, redo), the
   tabs on the left (Levels, Build, Catalog, Prefabs, Groups), the 3D view in the middle, the
   inspector of the selection on the right and a status bar. Its texts follow the launcher's
   language when the editor has it (English, French).
4. The editor speaks to its engine (Rust, running inside the launcher's process) through one Tauri
   command and reads the 3D data of the level through its own `jle://` protocol. The engine lives
   as long as the launcher: a saved level left open when the editor's window is closed is shown
   again when it is reopened; choosing **Don't save** when closing discards the document.
5. The first time, the editor needs the game's levels: **File > Extract game assets** reads Jak 3
   with `ogle-extract` (the editor's extractor, built from OpenGOAL's decompiler). In the launcher:
   - the source is the disc the launcher extracted when Jak 3 was installed
     (`<install>/active/jak3/data/iso_data/jak3`): the disc is not asked again;
   - the library goes by default in `<install>/jak-level-editor`;
   - `ogle-extract` is downloaded the first time from the release of the editor's version, into
     `<install>/jak-level-editor/tools/v<version>/`.

   The preferences are shared with the standalone editor (`%APPDATA%/OpenGOAL/jak-level-editor`).

6. After exporting a level for the game, **Play** in the editor brings the launcher's window to the
   front, which runs a job like a mod's installation, with its steps and its logs:
   1. **Download OpenGOAL** (the first time): the OpenGOAL the editor's levels need (the official
      versions lack the level builder's fixes for them), unpacked in
      `<install>/features/jak3/level_edited/`;
   2. **Decompile** (the first time): its extractor decompiles the disc the launcher already
      extracted;
   3. **Install Level**: the level's files are copied into that game's data and registered
      (`game.gp`, `level-info.gc`);
   4. **Compile**: the game is built with the level;
   5. **Play**: the game starts at the level's continue point, with its own settings and saves
      (`level_edited/_settings/`).

   The first Play takes about ten minutes (download, decompilation, a full build); the next ones
   only build again what changed. A build short of memory can stop on `bad allocation`: press
   Play again.

   ![The launcher's Play job: the runtime downloaded, the disc decompiled, the level installed, the game being built](../img/level_editor/capture_2.png)

7. On the Jak 3 page, **Advanced > Edited Levels** lists the levels installed in that game, with
   **Play** and **Remove** (the level is taken out and the game built again), and opens its folder.

   ![The Edited Levels page with one level installed](../img/level_editor/capture_3.png)

```mermaid
flowchart TD
  A["Sidebar: Level Editor button"] --> B["openLevelEditor(): the host refreshed, window 'level-editor'"]
  B --> C["App.svelte sees the label"]
  C --> D["LevelEditor component (level-editor/ui)"]
  D -->|"invoke plugin:jak-level-editor|call"| E["Rust engine (tauri-plugin-jak-level-editor)"]
  D -->|"jle:// protocol: meshes, textures"| E
  E --> F["Extracted levels, projects, exports"]
  E -->|"Play: GameRuntime.play"| G["event 'level_edited:play' to the main window"]
  G --> H["job playEditedLevel: download, decompile, install, compile, play"]
  H --> I["features/jak3/level_edited: gk at the level's continue point"]
```

## 3. How it Integrates into the Architecture

- **Source:** the editor's repository is a git submodule in `level-editor/`, pinned to a tag of the
  editor (the commit recorded by the launcher; no branch is followed). Clone the launcher with
  `git clone --recursive`, or run `git submodule update --init` once. To move to a new version of
  the editor:

  ```
  git -C level-editor fetch --tags
  git -C level-editor checkout vX.Y.Z
  git add level-editor
  git commit
  ```

- **Backend:** `src-tauri/Cargo.toml` depends on `level-editor/crates/tauri-plugin`; `main.rs`
  registers it with `.plugin(tauri_plugin_jak_level_editor::init())`. The plugin holds the editor's
  engine, its one command `call`, its `jle://` protocol and its remote control for AI assistants
  (port 47821: one editor at a time can hold it, so while the standalone editor runs, the
  launcher's says in its log that the remote control is unavailable).
- **What the launcher gives the editor** (`Host`, [`level-editor/crates/core/src/host.rs`](../../level-editor/crates/core/src/host.rs)):
  [`commands/features/level_edited.rs`](../../src-tauri/src/commands/features/level_edited.rs)
  builds it from the settings (`host()`); `main.rs` gives it with `set_host` once the settings are
  loaded, and `openLevelEditor` again each time the editor's window opens (command
  `refresh_level_editor_host`), so a new installation folder, a game installed or a downloaded
  extractor are seen. The editor uses what it is given and hides what it is not.

  | Field                | Value in the launcher                                                          |
  | :------------------- | :----------------------------------------------------------------------------- |
  | `name`               | `launcher`                                                                     |
  | `default_library`    | `<install>/jak-level-editor`                                                   |
  | `extract_source`     | `<install>/active/jak3/data/iso_data/jak3`, when Jak 3 is installed            |
  | `extractor`          | `<install>/jak-level-editor/tools/v<version>/ogle-extract(.exe)`, once fetched |
  | `extractor_provider` | downloads it (below)                                                           |
  | `runtime`            | the isolated game `<install>/features/jak3/level_edited/`                      |

- **Downloads:**
  - `ogle-extract`, the one of the editor's version (`EDITOR_VERSION` of the plugin):
    `https://github.com/whozghiar/open-goal-level-builder/releases/download/v<version>/ogle-extract-<platform>.zip`
    (`windows-x64`, `linux-x64`, `macos-x64`, `macos-arm64`; the archive holds `ogle-extract(.exe)`
    and `ogle-extract-data/`), unpacked in a `.part` folder renamed when complete, with its progress
    sent to the editor.
  - The game runtime: OpenGOAL with the level builder's fixes, from the release of the user's fork
    named at the top of `level_edited.rs` (`RUNTIME_VERSION`, `RUNTIME_RELEASE`), its archives
    named as OpenGOAL's own: `https://github.com/whozghiar/jak-project/releases/download/v0.3.9-jle.1/opengoal-windows-v0.3.9-jle.1.zip`
    (`opengoal-linux-…tar.gz`, `opengoal-macos-intel-…`, `opengoal-macos-arm-…` elsewhere). Its
    `data/custom_assets/jak3/jle-builder.json` (`{"builder": "jle-1"}`) says it has the builder the
    editor's levels need; without it the download is refused.
  - For tests, `JLE_EXTRACTOR_URL` and `JLE_RUNTIME_URL` replace them (a URL, or the path of a local
    archive).
- **The isolated game** `<install>/features/jak3/level_edited/`, like a mod: its own `gk`, `goalc`,
  `extractor` and `data/` (the decompiled game, the installed levels, the build), `_settings/`
  (`gk --config-path`: its settings and saves), and `jle-runtime.json` (the runtime's version, the
  disc decompiled). Vanilla `active/`, `versions/` and the mods are never touched. The editor's
  installer (`jle_core::io::level_package::apply` / `remove`) copies a level into `data/` and
  registers it, or takes it out.
- **Play:** the runtime's `play` emits `level_edited:play` (the export's folder, the level's name)
  to the main window and brings it to the front; `App.svelte` opens the job `playEditedLevel`
  ([`levelEditedJob.ts`](../../src/lib/job/levelEditedJob.ts)), whose steps call the commands
  `level_edited_prepare_runtime`, `level_edited_decompile`, `level_edited_install_level`,
  `level_edited_compile` and `level_edited_launch`. The launcher downloads and decompiles; the rest
  is the editor's own code (`jle_core::runtime::OpenGoal`, `jle_core::io::level_package`), which
  its standalone app uses too: the installation, the build's command line (`extractor --folder
--compile`), and the start at the continue point (`gk ... -- -boot -fakeiso -debug`, then
  goalc's REPL running `data/goal_src/user/jle/startup-jak3.gc`, stopped once the level is shown).
  The tools' output goes to the job's logs and `extractor-jak3-level_edited.log`, the game's to
  `level_edited/_settings/jle-game.log`. When the main window is closed, the runtime runs the same
  steps itself. The editor can also call `install_level`, `remove_level` and `launch` directly.
  **Remove** is the job `removeEditedLevel` (`level_edited_remove_level`, then compile; a game
  started by Play is stopped before any build).
- **Permissions:** the capability `jak-level-editor:default` and the `level-editor` window are added
  to `capabilities/main.json`, with the right to create a webview window.
- **Frontend:** Vite's alias `$level-editor` points to `level-editor/ui`; `App.svelte` mounts
  `<LevelEditor locale={config.locale} />` in the editor's window, as the standalone editor mounts
  it (with its own title bar, since the window has no system frame). The editor has its own
  translations (English, French) and follows the launcher's language. Tailwind scans
  `level-editor/ui` (`@source` in `style.css`); Svelte and Flowbite are deduplicated so the editor
  uses the launcher's copies. The **Edited Levels** page is the route `/:game_name/level_edited`.
- **Bundle:** the editor's texts and game tables are bundled under `jak-level-editor/`
  (`bundle.resources` in `tauri.conf.json`). `ogle-extract` is not bundled: it is downloaded (the
  editor's `JLE_EXTRACTOR` still gives it a path of your own).
- **CI:** every job of `.github/workflows/` that builds or type-checks the launcher checks out with
  `submodules: recursive`; `level-editor/` is in `.prettierignore` (the editor has its own
  formatting).

## 4. New Files Created

| File                                                                                                                             | Role                                                         |
| :------------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------------------- |
| [`.gitmodules`](../../.gitmodules), `level-editor/`                                                                              | the editor's repository as a submodule                       |
| [`src/lib/level-editor/window.ts`](../../src/lib/level-editor/window.ts)                                                         | opens (or focuses) the editor's window                       |
| [`src-tauri/src/commands/features/level_edited.rs`](../../src-tauri/src/commands/features/level_edited.rs)                       | the host, the extractor's download, the isolated game        |
| [`src/lib/job/levelEditedJob.ts`](../../src/lib/job/levelEditedJob.ts)                                                           | the steps of the jobs `playEditedLevel`, `removeEditedLevel` |
| [`src/components/level-edited/EditedLevels.svelte`](../../src/components/level-edited/EditedLevels.svelte)                       | the Edited Levels page                                       |
| [`docs/features_ai/features_level_editor.md`](features_level_editor.md)                                                          | this document                                                |
| [`docs/img/level_editor/capture_1.png`](../img/level_editor/capture_1.png)                                                       | the sidebar's button, for this document                      |
| [`docs/img/level_editor/capture_2.png`](../img/level_editor/capture_2.png), [`capture_3.png`](../img/level_editor/capture_3.png) | the Play job and the Edited Levels page, for this document   |

## 5. Overview of Changes from the Original Project

- [`src/components/sidebar/Sidebar.svelte`](../../src/components/sidebar/Sidebar.svelte): the
  **Level Editor** button, above Settings.
- [`src/App.svelte`](../../src/App.svelte): shows the editor, with its own title bar, instead of
  the router in the `level-editor` window; in the main window, opens the Play job on
  `level_edited:play`.
- [`src-tauri/Cargo.toml`](../../src-tauri/Cargo.toml), [`src-tauri/src/main.rs`](../../src-tauri/src/main.rs):
  the editor's plugin, its host, the `level_edited_*` commands.
- [`src-tauri/src/util/network.rs`](../../src-tauri/src/util/network.rs): `download_file_with_progress`
  (`download_file` uses it).
- [`src/routes/Job.svelte`](../../src/routes/Job.svelte), [`src/lib/job/jobs.ts`](../../src/lib/job/jobs.ts),
  [`src/lib/rpc/features.ts`](../../src/lib/rpc/features.ts): the two jobs and their commands.
- [`src/router.ts`](../../src/router.ts), [`src/components/games/GameControls.svelte`](../../src/components/games/GameControls.svelte):
  the Edited Levels page and its entry in Jak 3's **Advanced** menu.
- [`src-tauri/capabilities/main.json`](../../src-tauri/capabilities/main.json): its permission and
  window.
- [`src-tauri/tauri.conf.json`](../../src-tauri/tauri.conf.json): its resources in the bundle.
- [`vite.config.ts`](../../vite.config.ts), [`tsconfig.json`](../../tsconfig.json),
  [`src/css/style.css`](../../src/css/style.css): the `$level-editor` alias, deduplicated
  packages, Tailwind's scan.
- [`.github/workflows/`](../../.github/workflows/), [`.prettierignore`](../../.prettierignore): the
  submodule checked out in CI, left to its own formatting.
- [`src/assets/translations/en-US.json`](../../src/assets/translations/en-US.json): the button's
  label `sidebar_levelEditor`, the menu entry `gameControls_button_editedLevels`, the page and job
  texts `levelEdited_*`.

---

# 🇫🇷 Version Française

## 1. Qu'est-ce qu'elle apporte

Un bouton **Level Editor** dans la barre latérale ouvre **Jak Level Editor**, un éditeur de niveaux
pour Jak 3, dans sa propre fenêtre. On peut y ouvrir n'importe quel niveau du jeu (lu depuis son
propre disque), déplacer, copier et supprimer ses éléments, poser des objets, des préfabriqués et
des bâtiments entiers, peindre du terrain, voir le niveau éclairé comme le jeu l'éclaire à toute
heure, générer les chemins des créatures, et exporter le résultat comme un niveau personnalisé
qu'OpenGOAL compile et fait jouer. Un assistant IA peut aussi le piloter (serveur MCP).

Dans le launcher, l'éditeur fait aussi **jouer** ce qu'on construit : son bouton **Jouer** installe
le niveau dans un jeu à part, séparé de votre Jak 3, de ses sauvegardes et de vos mods, et l'y
lance, avec la page de progression du launcher. Le disque que le launcher a déjà extrait sert de
source à l'éditeur, et l'extracteur de l'éditeur est téléchargé la première fois qu'il sert.

L'éditeur est un projet à part ([open-goal-level-builder](https://github.com/whozghiar/open-goal-level-builder)),
écrit en Rust et Svelte comme le launcher. Le launcher se contente de le brancher : il ne garde
aucune copie de son code.

## 2. Comment fonctionne la fonctionnalité

1. L'utilisateur clique sur **Level Editor** (icône crayon et règle) dans la barre latérale.

   ![Barre latérale du launcher avec le bouton Level Editor et son infobulle](../img/level_editor/capture_1.png)

2. Le launcher ouvre une seconde fenêtre nommée `level-editor`, agrandie, ou la ramène au premier
   plan (restaurée si elle était réduite) si elle est déjà ouverte. Fermer la fenêtre principale du
   launcher laisse l'éditeur ouvert ; un travail non enregistré n'est jamais perdu en silence
   (l'éditeur demande avant de fermer).
3. Cette fenêtre charge l'interface du launcher, qui reconnaît son nom et affiche l'éditeur à la
   place des pages du launcher. Comme la fenêtre du launcher, elle n'a pas de cadre système :
   l'éditeur dessine sa propre barre de titre (le nom du niveau, un point orange s'il n'est pas
   enregistré, réduire, agrandir, fermer). Dessous viennent les menus (Fichier, Édition, Affichage,
   Construire, Niveau, Aide), la barre d'outils (déplacer, pivoter, échelle, topologie, terrain,
   copier, coller, supprimer, créer un préfabriqué, annuler, rétablir), les onglets à gauche
   (Niveaux, Construire, Catalogue, Préfabriqués, Groupes), la vue 3D au centre, l'inspecteur de la
   sélection à droite et une barre d'état. Ses textes suivent la langue du launcher quand
   l'éditeur la connaît (anglais, français).
4. L'éditeur parle à son moteur (Rust, dans le processus du launcher) par une seule commande Tauri
   et lit les données 3D du niveau par son propre protocole `jle://`. Le moteur vit aussi longtemps
   que le launcher : un niveau enregistré resté ouvert quand on ferme la fenêtre de l'éditeur
   réapparaît quand on la rouvre ; choisir **Ne pas enregistrer** en fermant abandonne le document.
5. La première fois, l'éditeur a besoin des niveaux du jeu : **Fichier > Extraire les données du
   jeu** lit Jak 3 avec `ogle-extract` (l'extracteur de l'éditeur, tiré du décompilateur
   d'OpenGOAL). Dans le launcher :
   - la source est le disque que le launcher a extrait à l'installation de Jak 3
     (`<install>/active/jak3/data/iso_data/jak3`) : le disque n'est pas redemandé ;
   - la bibliothèque va par défaut dans `<install>/jak-level-editor` ;
   - `ogle-extract` est téléchargé la première fois depuis la release de la version de l'éditeur,
     dans `<install>/jak-level-editor/tools/v<version>/`.

   Les préférences sont partagées avec l'éditeur autonome (`%APPDATA%/OpenGOAL/jak-level-editor`).

6. Après avoir exporté un niveau pour le jeu, **Jouer** dans l'éditeur ramène la fenêtre du
   launcher au premier plan, qui lance une tâche comme l'installation d'un mod, avec ses étapes et
   ses journaux :
   1. **Télécharger OpenGOAL** (la première fois) : l'OpenGOAL dont les niveaux de l'éditeur ont
      besoin (les versions officielles n'ont pas les corrections du constructeur de niveaux pour
      eux), décompressé dans `<install>/features/jak3/level_edited/` ;
   2. **Décompiler** (la première fois) : son extracteur décompile le disque que le launcher a déjà
      extrait ;
   3. **Installer le niveau** : les fichiers du niveau sont copiés dans les données de ce jeu et
      déclarés (`game.gp`, `level-info.gc`) ;
   4. **Compiler** : le jeu est construit avec le niveau ;
   5. **Jouer** : le jeu démarre au point de reprise du niveau, avec ses propres réglages et
      sauvegardes (`level_edited/_settings/`).

   Le premier Jouer prend une dizaine de minutes (téléchargement, décompilation, compilation
   complète) ; les suivants ne recompilent que ce qui a changé. Une compilation à court de mémoire
   peut s'arrêter sur `bad allocation` : relancer Jouer.

   ![La tâche Jouer du launcher : moteur téléchargé, disque décompilé, niveau installé, jeu en cours de compilation](../img/level_editor/capture_2.png)

7. Sur la page de Jak 3, **Advanced > Edited Levels** liste les niveaux installés dans ce jeu, avec
   **Play** et **Remove** (le niveau est retiré et le jeu reconstruit), et ouvre son dossier.

   ![La page Edited Levels avec un niveau installé](../img/level_editor/capture_3.png)

```mermaid
flowchart TD
  A["Barre latérale : bouton Level Editor"] --> B["openLevelEditor() : hôte rafraîchi, fenêtre 'level-editor'"]
  B --> C["App.svelte reconnaît la fenêtre"]
  C --> D["Composant LevelEditor (level-editor/ui)"]
  D -->|"invoke plugin:jak-level-editor|call"| E["Moteur Rust (tauri-plugin-jak-level-editor)"]
  D -->|"protocole jle:// : maillages, textures"| E
  E --> F["Niveaux extraits, projets, exports"]
  E -->|"Jouer : GameRuntime.play"| G["événement 'level_edited:play' vers la fenêtre principale"]
  G --> H["tâche playEditedLevel : télécharger, décompiler, installer, compiler, jouer"]
  H --> I["features/jak3/level_edited : gk au point de reprise du niveau"]
```

## 3. Comment elle s'intègre dans l'architecture

- **Source :** le dépôt de l'éditeur est un sous-module git dans `level-editor/`, fixé sur un tag de
  l'éditeur (le commit enregistré par le launcher ; aucune branche n'est suivie). Cloner le
  launcher avec `git clone --recursive`, ou lancer une fois `git submodule update --init`. Pour
  passer à une nouvelle version de l'éditeur :

  ```
  git -C level-editor fetch --tags
  git -C level-editor checkout vX.Y.Z
  git add level-editor
  git commit
  ```

- **Backend :** `src-tauri/Cargo.toml` dépend de `level-editor/crates/tauri-plugin` ; `main.rs`
  l'enregistre avec `.plugin(tauri_plugin_jak_level_editor::init())`. Le plugin contient le moteur
  de l'éditeur, son unique commande `call`, son protocole `jle://` et sa télécommande pour les
  assistants IA (port 47821 : un seul éditeur à la fois peut l'occuper, donc tant que l'éditeur
  autonome tourne, celui du launcher indique dans son journal que la télécommande est
  indisponible).
- **Ce que le launcher donne à l'éditeur** (`Host`, [`level-editor/crates/core/src/host.rs`](../../level-editor/crates/core/src/host.rs)) :
  [`commands/features/level_edited.rs`](../../src-tauri/src/commands/features/level_edited.rs) le
  construit à partir des réglages (`host()`) ; `main.rs` le donne avec `set_host` une fois les
  réglages chargés, et `openLevelEditor` à nouveau à chaque ouverture de la fenêtre de l'éditeur
  (commande `refresh_level_editor_host`), pour qu'un nouveau dossier d'installation, un jeu
  installé ou un extracteur téléchargé soient pris en compte. L'éditeur utilise ce qu'on lui donne
  et cache le reste.

  | Champ                | Valeur dans le launcher                                                           |
  | :------------------- | :-------------------------------------------------------------------------------- |
  | `name`               | `launcher`                                                                        |
  | `default_library`    | `<install>/jak-level-editor`                                                      |
  | `extract_source`     | `<install>/active/jak3/data/iso_data/jak3`, quand Jak 3 est installé              |
  | `extractor`          | `<install>/jak-level-editor/tools/v<version>/ogle-extract(.exe)`, une fois obtenu |
  | `extractor_provider` | le télécharge (ci-dessous)                                                        |
  | `runtime`            | le jeu à part `<install>/features/jak3/level_edited/`                             |

- **Téléchargements :**
  - `ogle-extract`, celui de la version de l'éditeur (`EDITOR_VERSION` du plugin) :
    `https://github.com/whozghiar/open-goal-level-builder/releases/download/v<version>/ogle-extract-<plateforme>.zip`
    (`windows-x64`, `linux-x64`, `macos-x64`, `macos-arm64` ; l'archive contient
    `ogle-extract(.exe)` et `ogle-extract-data/`), décompressé dans un dossier `.part` renommé une
    fois complet, sa progression envoyée à l'éditeur.
  - Le moteur du jeu : OpenGOAL avec les corrections du constructeur de niveaux, depuis la release
    du fork de l'utilisateur indiquée en tête de `level_edited.rs` (`RUNTIME_VERSION`,
    `RUNTIME_RELEASE`), ses archives nommées comme celles d'OpenGOAL :
    `https://github.com/whozghiar/jak-project/releases/download/v0.3.9-jle.1/opengoal-windows-v0.3.9-jle.1.zip`
    (`opengoal-linux-…tar.gz`, `opengoal-macos-intel-…`, `opengoal-macos-arm-…` ailleurs). Son
    `data/custom_assets/jak3/jle-builder.json` (`{"builder": "jle-1"}`) indique qu'il a le
    constructeur dont les niveaux de l'éditeur ont besoin ; sans lui le téléchargement est refusé.
  - Pour les tests, `JLE_EXTRACTOR_URL` et `JLE_RUNTIME_URL` les remplacent (une URL, ou le chemin
    d'une archive locale).
- **Le jeu à part** `<install>/features/jak3/level_edited/`, comme un mod : ses propres `gk`,
  `goalc`, `extractor` et `data/` (le jeu décompilé, les niveaux installés, la compilation),
  `_settings/` (`gk --config-path` : ses réglages et sauvegardes), et `jle-runtime.json` (la version
  du moteur, le disque décompilé). Le `active/` du jeu de base, `versions/` et les mods ne sont
  jamais touchés. L'installateur de l'éditeur (`jle_core::io::level_package::apply` / `remove`)
  copie un niveau dans `data/` et le déclare, ou le retire.
- **Jouer :** le `play` du moteur émet `level_edited:play` (le dossier de l'export, le nom du
  niveau) vers la fenêtre principale et la ramène au premier plan ; `App.svelte` ouvre la tâche
  `playEditedLevel` ([`levelEditedJob.ts`](../../src/lib/job/levelEditedJob.ts)), dont les étapes
  appellent les commandes `level_edited_prepare_runtime`, `level_edited_decompile`,
  `level_edited_install_level`, `level_edited_compile` et `level_edited_launch`. Le launcher
  télécharge et décompile ; le reste est le code de l'éditeur lui-même (`jle_core::runtime::OpenGoal`,
  `jle_core::io::level_package`), que son application autonome utilise aussi : l'installation, la
  ligne de commande de la compilation (`extractor --folder --compile`), et le démarrage au point de
  reprise (`gk ... -- -boot -fakeiso -debug`, puis le REPL de goalc qui exécute
  `data/goal_src/user/jle/startup-jak3.gc`, arrêté une fois le niveau affiché). La sortie des outils
  va dans les journaux de la tâche et `extractor-jak3-level_edited.log`, celle du jeu dans
  `level_edited/_settings/jle-game.log`. Quand la fenêtre principale est fermée, le moteur fait
  lui-même les mêmes étapes. L'éditeur peut aussi appeler directement `install_level`,
  `remove_level` et `launch`. **Remove** est la tâche `removeEditedLevel`
  (`level_edited_remove_level`, puis compilation ; un jeu lancé par Jouer est arrêté avant toute
  compilation).
- **Permissions :** la capacité `jak-level-editor:default` et la fenêtre `level-editor` sont ajoutées
  à `capabilities/main.json`, avec le droit de créer une fenêtre webview.
- **Frontend :** l'alias Vite `$level-editor` pointe vers `level-editor/ui` ; `App.svelte` affiche
  `<LevelEditor locale={config.locale} />` dans la fenêtre de l'éditeur, comme l'éditeur autonome
  le monte (avec sa propre barre de titre, puisque la fenêtre n'a pas de cadre système). L'éditeur
  a ses propres traductions (anglais, français) et suit la langue du launcher. Tailwind analyse
  `level-editor/ui` (`@source` dans `style.css`) ; Svelte et Flowbite sont dédupliqués pour que
  l'éditeur utilise les copies du launcher. La page **Edited Levels** est la route
  `/:game_name/level_edited`.
- **Paquet :** les textes et tables du jeu de l'éditeur sont embarqués sous `jak-level-editor/`
  (`bundle.resources` dans `tauri.conf.json`). `ogle-extract` n'est pas embarqué : il est téléchargé
  (le `JLE_EXTRACTOR` de l'éditeur permet toujours d'en donner un à soi).
- **CI :** chaque tâche de `.github/workflows/` qui compile ou vérifie les types du launcher fait son
  checkout avec `submodules: recursive` ; `level-editor/` est dans `.prettierignore` (l'éditeur a
  son propre formatage).

## 4. Quels sont les nouveaux fichiers

| Fichier                                                                                                                          | Rôle                                                         |
| :------------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------------------- |
| [`.gitmodules`](../../.gitmodules), `level-editor/`                                                                              | le dépôt de l'éditeur en sous-module                         |
| [`src/lib/level-editor/window.ts`](../../src/lib/level-editor/window.ts)                                                         | ouvre (ou met au premier plan) la fenêtre de l'éditeur       |
| [`src-tauri/src/commands/features/level_edited.rs`](../../src-tauri/src/commands/features/level_edited.rs)                       | l'hôte, le téléchargement de l'extracteur, le jeu à part     |
| [`src/lib/job/levelEditedJob.ts`](../../src/lib/job/levelEditedJob.ts)                                                           | les étapes des tâches `playEditedLevel`, `removeEditedLevel` |
| [`src/components/level-edited/EditedLevels.svelte`](../../src/components/level-edited/EditedLevels.svelte)                       | la page Edited Levels                                        |
| [`docs/features_ai/features_level_editor.md`](features_level_editor.md)                                                          | ce document                                                  |
| [`docs/img/level_editor/capture_1.png`](../img/level_editor/capture_1.png)                                                       | le bouton de la barre latérale, pour ce document             |
| [`docs/img/level_editor/capture_2.png`](../img/level_editor/capture_2.png), [`capture_3.png`](../img/level_editor/capture_3.png) | la tâche Jouer et la page Edited Levels, pour ce document    |

## 5. Quels sont les modifications dans les grandes lignes

- [`src/components/sidebar/Sidebar.svelte`](../../src/components/sidebar/Sidebar.svelte) : le bouton
  **Level Editor**, au-dessus des Paramètres.
- [`src/App.svelte`](../../src/App.svelte) : affiche l'éditeur, avec sa propre barre de titre, à la
  place du routeur dans la fenêtre `level-editor` ; dans la fenêtre principale, ouvre la tâche
  Jouer sur `level_edited:play`.
- [`src-tauri/Cargo.toml`](../../src-tauri/Cargo.toml), [`src-tauri/src/main.rs`](../../src-tauri/src/main.rs) :
  le plugin de l'éditeur, son hôte, les commandes `level_edited_*`.
- [`src-tauri/src/util/network.rs`](../../src-tauri/src/util/network.rs) : `download_file_with_progress`
  (`download_file` l'utilise).
- [`src/routes/Job.svelte`](../../src/routes/Job.svelte), [`src/lib/job/jobs.ts`](../../src/lib/job/jobs.ts),
  [`src/lib/rpc/features.ts`](../../src/lib/rpc/features.ts) : les deux tâches et leurs commandes.
- [`src/router.ts`](../../src/router.ts), [`src/components/games/GameControls.svelte`](../../src/components/games/GameControls.svelte) :
  la page Edited Levels et son entrée dans le menu **Advanced** de Jak 3.
- [`src-tauri/capabilities/main.json`](../../src-tauri/capabilities/main.json) : sa permission et sa
  fenêtre.
- [`src-tauri/tauri.conf.json`](../../src-tauri/tauri.conf.json) : ses ressources dans le paquet.
- [`vite.config.ts`](../../vite.config.ts), [`tsconfig.json`](../../tsconfig.json),
  [`src/css/style.css`](../../src/css/style.css) : l'alias `$level-editor`, les paquets dédupliqués,
  l'analyse de Tailwind.
- [`.github/workflows/`](../../.github/workflows/), [`.prettierignore`](../../.prettierignore) : le
  sous-module récupéré en CI, laissé à son propre formatage.
- [`src/assets/translations/en-US.json`](../../src/assets/translations/en-US.json) : le libellé du
  bouton `sidebar_levelEditor`, l'entrée de menu `gameControls_button_editedLevels`, les textes de
  la page et des tâches `levelEdited_*`.
