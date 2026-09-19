> **Language / Langue :** [🇬🇧 English Version](#-english-version) &nbsp;•&nbsp; [🇫🇷 Version Française](#-version-française)

## Summary / Sommaire

- [🇬🇧 English Version](#-english-version)
  - [1. Overview & Architecture](#1-overview--architecture)
  - [2. Local Environment & Windows Tooling](#2-local-environment--windows-tooling)
  - [3. AI Assistant Rules & Behavioral Protocols](#3-ai-assistant-rules--behavioral-protocols)
  - [4. Modding & Runtime Pipeline](#4-modding--runtime-pipeline)
  - [5. AI Features Documentation Standard (`docs/features_ai/`)](#5-ai-features-documentation-standard-docsfeatures_ai)
  - [6. Documentation Standard](#6-documentation-standard)
- [🇫🇷 Version Française](#-version-française)
  - [1. Vue d'ensemble & Architecture](#1-vue-densemble--architecture)
  - [2. Environnement Local & Outillage Windows](#2-environnement-local--outillage-windows)
  - [3. Règles de l'Assistant IA & Protocoles](#3-règles-de-lassistant-ia--protocoles)
  - [4. Pipeline d'Exécution & Modding](#4-pipeline-dexécution--modding)
  - [5. Standard de Documentation des Fonctionnalités IA (`docs/features_ai/`)](#5-standard-de-documentation-des-fonctionnalités-ia-docsfeatures_ai)
  - [6. Standard de Documentation Bilingue](#6-standard-de-documentation-bilingue)

---

# 🇬🇧 English Version

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
    - `game.rs`: Active game detection and save milestone tracking.
    - `support.rs`: Support package generation (logs, config, hardware diagnostics).

---

## 2. Local Environment & Windows Tooling

On this Windows development environment, build tools and runtimes are located at the following exact paths:

| Tool                       | Path / Command                                                   | Notes                                                             |
| :------------------------- | :--------------------------------------------------------------- | :---------------------------------------------------------------- |
| **Rust Toolchain**         | `C:\Users\IRLANDO\.cargo\bin\cargo.exe`                          | Used for compiling and running tests in `src-tauri`.              |
| **Node.js Package Runner** | `npx yarn`                                                       | Runs workspace scripts without requiring a global Yarn install.   |
| **Frontend Dev Server**    | `npx yarn dev`                                                   | Starts Vite for UI browser testing.                               |
| **Desktop Dev Server**     | `npx yarn tauri dev`                                             | Launches full Tauri application with live reload (Rust + Svelte). |
| **Typecheck**              | `npx yarn typecheck`                                             | Runs `svelte-check` across Svelte 5 and TypeScript files.         |
| **Code Formatting**        | `npx yarn check-format` / `npx yarn format`                      | Prettier with Svelte plugin.                                      |
| **Rust Tests**             | `cd src-tauri && & "C:\Users\IRLANDO\.cargo\bin\cargo.exe" test` | Runs Rust backend tests and binding export tests.                 |

---

## 3. AI Assistant Rules & Behavioral Protocols

When assisting on this repository, the AI must strictly adhere to the following mandatory constraints:

1. **Explicit User Notification on File Changes**:
   The AI must **never** create, modify, or delete any file without explicitly informing the user beforehand or clearly describing the modification in its response. Unreported file alterations are strictly prohibited.
2. **Strict Adherence to Initial Project Philosophy**:
   The AI must scrupulously respect the project's original philosophy, architectural separation, semantics, and syntax to minimize regressions. Never introduce unnecessary changes to unrelated files, and refactor or delete any superfluous code.
3. **Simple Comments Above Functions**:
   Always place a short, simple, single-sentence comment directly above each newly added or modified function explaining what it does and what its purpose is. Keep comments direct, concise, and focused on intent.
4. **Strict Prohibition on Editing Generated Files**:
   Files that explicitly state not to edit them manually (such as `// This file was generated by [ts-rs]... Do not edit this file manually`) must **never** be manually edited. Updates must be performed solely through the official mechanism/tooling designated for those files.
5. **Bilingual, Jargon-Free Commit Messages**:
   All commit messages must be provided in both English and French (`<English description> / <French description>`). They must be short, clear, explicit, and avoid repository-specific or excessively obscure technical jargon.
6. **Mandatory AI-Assisted Suffix**:
   All commits produced with AI assistance must end with the suffix `(AI-assisted)`.
   - _Example_: `feat: allow sharing vanilla saves with mods / autoriser le partage des sauvegardes du jeu de base avec les mods (AI-assisted)`
7. **Mandatory Bilingual Feature Documentation (`docs/features_ai/`)**:
   For every new feature implemented with AI assistance, create a dedicated bilingual markdown file named `features_[feature_name].md` inside `docs/features_ai/` adhering to the guidelines in Section 5.

---

## 4. Modding & Runtime Pipeline

1. **Mod Sources**: JSON feeds (defined in `schemas/mod-source/v1/`) declaring mods, versions, download links, and supported games (`jak1`, `jak2`, `jak3`, `jakx`).
2. **Directory Layout**:
   - Installed mods: `<install_dir>/features/<game>/mods/<source>/<mod_name>/`.
   - Mod settings & isolated saves: `<install_dir>/features/<game>/mods/<source>/_settings/<mod_name>/`.
3. **Execution Engine (`gk`)**:
   - Vanilla game: `gk` saves data to `%APPDATA%/OpenGOAL/<game>/saves` (Windows) or `~/.config/OpenGOAL/<game>/saves` (Linux).
   - Mods: launched with `--config-path <mod_settings_dir>`, which isolates both settings and save directories.
   - Shared saves: passing `--disable_save_location_override` alongside `--config-path` directs save files to the vanilla saves folder while preserving isolated mod settings.

---

## 5. AI Features Documentation Standard (`docs/features_ai/`)

For every new feature developed with AI assistance, a dedicated bilingual documentation file must be created:

```
docs/features_ai/features_[feature_name].md
```

### Educational Approach & Writing Style

- **Accessible to Everyone**: Written in an educational and clear manner that is easy to understand for developers as well as non-technical readers or beginners.
- **Direct & Clear**: Strictly limit superfluous phrases, buzzwords, and repetitive boilerplate typically produced by AI. Be concise, direct, and focused.
- **Bilingual Structure**: Must implement the strict bilingual standard (top switcher, mini-summary, `# 🇬🇧 English Version`, `# 🇫🇷 Version Française`).
- **Relative Git Links**: All links to codebase files must use relative paths (e.g. `../../src/...`) so they work seamlessly when viewed on Git/GitHub.
- **Clean Mermaid Diagrams**: If architecture flows are illustrated, use valid and robust `flowchart TD` syntax with quoted labels.

### Mandatory Content Sections

Each feature document must address the following 5 points:

1. **What this feature brings**: User-facing value and functional enhancements.
2. **How the feature works**: Plain-language explanation of the lifecycle and process flow.
3. **How it integrates into the architecture**: Overview of how the feature fits across frontend, backend, and game execution.
4. **New files created**: List and concise description of each new file added to the codebase.
5. **Overview of changes from the original project**: High-level summary of what was adjusted in existing code, without overwhelming the reader with minor technical trivia.

---

## 6. Documentation Standard

All `.md` files must follow this template:

```markdown
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

...

---

# 🇫🇷 Version Française

...
```

---

# 🇫🇷 Version Française

## 1. Vue d'ensemble & Architecture

L'**OpenGOAL Launcher** est une application de bureau multiplateforme conçue pour installer, gérer, mettre à jour et lancer les versions officielles d'OpenGOAL, les packs de textures et les mods communautaires pour la trilogie _Jak and Daxter_.

### Pile Technologique

- **Framework Desktop** : [Tauri v2](https://tauri.app/) (moteur natif Rust et bus de communication IPC).
- **Frontend** : [Svelte 5](https://svelte.dev/) (utilisant les runes modernes : `$state`, `$derived`, `$props`, `$effect`).
- **Styles** : [Tailwind CSS v4](https://tailwindcss.com/) & [Flowbite Svelte](https://flowbite-svelte.com/).
- **Bundler & Outillage** : [Vite](https://vitejs.dev/) avec TypeScript.
- **Architecture Backend (`src-tauri/`)** :
  - `src-tauri/src/main.rs` : Point d'entrée de l'application, enregistrement des plugins et routage des commandes IPC.
  - `src-tauri/src/config.rs` : Modèle `LauncherConfig`, sérialisation sur disque (`settings.json`), migrations de versions et résolutions de chemins.
  - `src-tauri/src/cache.rs` : Cache des métadonnées de mods, schémas distants et compteurs de téléchargements.
  - `src-tauri/src/commands/` : Commandes Tauri organisées par domaine métier :
    - `binaries.rs` : Compilation, extraction des ISOs et lancement du jeu officiel (`gk`).
    - `features/mods.rs` : Téléchargement, extraction des archives, construction des arguments CLI et exécution des mods.
    - `features/texture_packs.rs` : Gestion et installation des packs de textures HD.
    - `game.rs` : Détection du jeu actif et lecture des jalons de sauvegarde.
    - `support.rs` : Génération du package de support (logs, configuration, métriques système).

---

## 2. Environnement Local & Outillage Windows

Sous Windows, les outils de développement et gestionnaires de paquets sont configurés selon les chemins suivants :

| Outil                           | Exécutable / Commande                                            | Remarques                                                            |
| :------------------------------ | :--------------------------------------------------------------- | :------------------------------------------------------------------- |
| **Rust Toolchain**              | `C:\Users\IRLANDO\.cargo\bin\cargo.exe`                          | Utilisé pour compiler et tester `src-tauri`.                         |
| **Node.js Package Runner**      | `npx yarn`                                                       | Exécute les scripts du projet sans installation globale de Yarn.     |
| **Serveur Dev Frontend**        | `npx yarn dev`                                                   | Démarre Vite pour développer l'interface Svelte dans le navigateur.  |
| **Serveur Dev Desktop Complet** | `npx yarn tauri dev`                                             | Lance l'application Tauri avec rechargement à chaud (Rust + Svelte). |
| **Vérification des Types**      | `npx yarn typecheck`                                             | Analyse TypeScript et les composants Svelte 5 (`svelte-check`).      |
| **Formatage de Code**           | `npx yarn check-format` / `npx yarn format`                      | Prettier configuré avec le plugin Svelte.                            |
| **Tests Rust**                  | `cd src-tauri && & "C:\Users\IRLANDO\.cargo\bin\cargo.exe" test` | Exécute les tests unitaires du backend Rust et l'export des types.   |

---

## 3. Règles de l'Assistant IA & Protocoles

Lors de toute intervention sur ce dépôt, l'IA doit impérativement respecter les règles suivantes :

1. **Information Préalable et Transparence Totale** :
   L'IA ne doit **jamais** créer, modifier ou supprimer un fichier sans en informer explicitement l'utilisateur ou sans détailler précisément la modification dans sa réponse. Les altérations silencieuses de fichiers sont strictement interdites.
2. **Respect Impératif de la Philosophie Initiale du Projet** :
   L'IA doit scrupuleusement respecter la philosophie, l'architecture, la sémantique et la syntaxe d'origine du projet afin de limiter les régressions. Ne jamais modifier inutilement des fichiers non liés, et refactoriser ou supprimer tout code inutilement ajouté.
3. **Commentaire Simple au-dessus des Fonctions** :
   Laisser impérativement un commentaire simple, court et clair (une phrase) directement au-dessus de chaque fonction créée ou modifiée pour expliquer à quoi elle sert et son intention.
4. **Interdiction Formelle de Modifier les Fichiers Générés** :
   Les fichiers comportant la mention explicite de ne pas les modifier manuellement (par exemple : `// This file was generated by [ts-rs]... Do not edit this file manually`) ne doivent **en aucun cas** être édités manuellement. Toute mise à jour doit se faire exclusivement par le biais de l'outil ou processus officiel prévu à cet effet.
5. **Messages de Commit Bilingues et Accessibles** :
   Les messages de commit doivent obligatoirement être rédigés en anglais et en français (`<description en anglais> / <description en français>`). Ils doivent être courts, explicites et dépourvus de jargon interne trop complexe.
6. **Suffixe Obligatoire pour les Commits IA** :
   Tout commit assisté ou généré via l'IA doit obligatoirement se terminer par le suffixe `(AI-assisted)`.
   - _Exemple_ : `feat: allow sharing vanilla saves with mods / autoriser le partage des sauvegardes du jeu de base avec les mods (AI-assisted)`
7. **Documentation Obligatoire des Fonctionnalités IA (`docs/features_ai/`)** :
   Pour chaque nouvelle fonctionnalité développée avec l'assistance de l'IA, créer un fichier Markdown bilingue dédié préfixé par `features_[nom_feature].md` dans `docs/features_ai/` selon les directives de la Section 5.

---

## 4. Pipeline d'Exécution & Modding

1. **Sources de mods** : Catalogues JSON (spécifiés dans `schemas/mod-source/v1/`) déclarant les mods, leurs versions, liens de téléchargement d'archives et jeux supportés (`jak1`, `jak2`, `jak3`, `jakx`).
2. **Arborescence des dossiers** :
   - Mods installés : `<install_dir>/features/<game>/mods/<source>/<mod_name>/`.
   - Paramètres et sauvegardes isolées des mods : `<install_dir>/features/<game>/mods/<source>/_settings/<mod_name>/`.
3. **Moteur d'exécution (`gk`)** :
   - Jeu de base (vanilla) : `gk` enregistre dans `%APPDATA%/OpenGOAL/<game>/saves` (Windows) ou `~/.config/OpenGOAL/<game>/saves` (Linux).
   - Mods : exécutés avec `--config-path <mod_settings_dir>`, isolant réglages et sauvegardes.
   - Sauvegardes partagées : l'argument `--disable_save_location_override` combiné à `--config-path` permet d'utiliser le dossier de sauvegardes vanilla tout en préservant des paramètres de mod isolés.

---

## 5. Standard de Documentation des Fonctionnalités IA (`docs/features_ai/`)

Pour chaque nouvelle fonctionnalité développée avec l'assistance de l'IA, un fichier de documentation bilingue dédié doit être créé :

```
docs/features_ai/features_[nom_feature].md
```

### Démarche Pédagogique & Style Rédactionnel

- **Accessible à Tous** : Rédaction pédagogique, compréhensible aussi bien par un développeur (même inexpérimenté) que par un non-initié.
- **Direct & Clair** : Limiter impérativement les phrases et mots superflus habituellement posés par l'IA. Aller droit au but, être concis et limpide.
- **Structure Bilingue** : Implémenter obligatoirement le format double version (bandeau de commutation, mini-sommaire, section `# 🇬🇧 English Version` et section `# 🇫🇷 Version Française`).
- **Liens Relatifs Git** : Tous les liens vers des fichiers du dépôt doivent utiliser des chemins relatifs (ex. `../../src/...`) pour fonctionner immédiatement lors de la navigation sur Git/GitHub.
- **Diagrammes Mermaid Propres** : Si des flux sont décrits, utiliser une syntaxe `flowchart TD` propre avec des libellés entre guillemets pour éviter toute erreur d'interprétation.

### Contenu Obligatoire en 5 Points

Chaque document de fonctionnalité doit obligatoirement détailler :

1. **Qu'est-ce qu'elle apporte** : Valeur ajoutée, gains utilisateurs et fonctionnalités offertes.
2. **Comment fonctionne la fonctionnalité** : Parcours concret et cycle de vie expliqués simplement.
3. **Comment elle s'intègre dans l'architecture** : Articulation entre le frontend, le backend Tauri et le moteur de jeu.
4. **Quels sont les nouveaux fichiers** : Liste de chaque nouveau fichier créé et son rôle en une ou deux phrases.
5. **Quels sont les modifications dans les grandes lignes** : Synthèse des modifications apportées par rapport au projet initial, sans noyer le lecteur sous des détails techniques secondaires.

---

## 6. Standard de Documentation Bilingue

Chaque fichier `.md` de fonctionnalité doit suivre ce modèle :

```markdown
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

...

---

# 🇫🇷 Version Française

...
```
