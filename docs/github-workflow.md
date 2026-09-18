> **Language / Langue :** [🇬🇧 English Version](#-english-version) &nbsp;•&nbsp; [🇫🇷 Version Française](#-version-française)

## Summary / Sommaire

- [🇬🇧 English Version](#-english-version)
  - [1. Introduction: What is CI/CD and GitHub Actions?](#1-introduction-what-is-cicd-and-github-actions)
  - [2. Overview of the 4 Workflows](#2-overview-of-the-4-workflows)
  - [3. Workflow Details](#3-workflow-details)
    - [🔨 Build (`build.yaml`)](#-build-buildyaml)
    - [📝 Linter (`lint.yaml`)](#-linter-lintyaml)
    - [🧪 Tests (`test.yaml`)](#-tests-testyaml)
    - [🏭 Create Release (`release.yaml`)](#-create-release-releaseyaml)
  - [4. Summary Table](#4-summary-table)
- [🇫🇷 Version Française](#-version-française)
  - [1. Introduction : Qu'est-ce que la CI/CD et les GitHub Actions ?](#1-introduction--quest-ce-que-la-cicd-et-les-github-actions-)
  - [2. Vue d'ensemble des 4 Workflows](#2-vue-densemble-des-4-workflows)
  - [3. Détail de chaque Workflow](#3-détail-de-chaque-workflow)
    - [🔨 Build (`build.yaml`)](#-build-buildyaml-1)
    - [📝 Linter (`lint.yaml`)](#-linter-lintyaml-1)
    - [🧪 Tests (`test.yaml`)](#-tests-testyaml-1)
    - [🏭 Création de Version / Release (`release.yaml`)](#-création-de-version--release-releaseyaml)
  - [4. Tableau Récapitulatif](#4-tableau-récapitulatif)

---

# 🇬🇧 English Version

## 1. Introduction: What is CI/CD and GitHub Actions?

When multiple developers work together on software, checking that the code compiles, works properly, and follows formatting standards by hand on every computer is slow and prone to human error.

**GitHub Actions** is an automated assistant built into GitHub. Think of it like an automated digital assembly line:

- Whenever code is pushed or a Pull Request is submitted, GitHub launches clean virtual computers in the cloud (Linux, Windows, macOS).
- It runs automated checks (**CI** - _Continuous Integration_), such as testing the code and checking formatting.
- When ready, it can package the final installers for users (**CD** - _Continuous Delivery_).

In this repository, all automated pipelines are stored as YAML configuration files in the `.github/workflows/` directory.

---

## 2. Overview of the 4 Workflows

| Workflow File  | Name              | Purpose                                             | When does it run?                        |
| :------------- | :---------------- | :-------------------------------------------------- | :--------------------------------------- |
| `build.yaml`   | 🔨 Build          | Compiles the launcher on Windows, Linux, and macOS  | On pushes/PRs to `main` and release tags |
| `lint.yaml`    | 📝 Linter         | Verifies code styling, syntax formatting, and types | On pushes and PRs to `main`              |
| `test.yaml`    | 🧪 Tests          | Executes automated unit and component tests         | On pushes/PRs to `main` and release tags |
| `release.yaml` | 🏭 Create Release | Builds production installers and publishes updates  | Triggered manually by a maintainer       |

---

## 3. Workflow Details

### 🔨 Build (`build.yaml`)

**Goal**: Verify that the application successfully compiles into an executable on all supported operating systems without any compile errors.

- **Matrix**: Runs on 4 distinct virtual machines in parallel:
  1. `ubuntu-24.04` (Linux)
  2. `windows-latest` (Windows)
  3. `macos-15` (Apple Silicon Mac)
  4. `macos-15-intel` (Intel Mac)
- **Key Steps**:
  1. Sets up the **Node.js** (v24) and **Rust** environments.
  2. Restores previously cached build files to speed up compilation (`rust-cache`).
  3. On Linux, installs required graphical libraries (`libwebkit2gtk`, `libxdo`, `libayatana-appindicator3`).
  4. Runs `yarn install` and builds the frontend (`yarn build`).
  5. Runs Tauri's build action to create the desktop executable and application bundles (e.g. Linux `.AppImage`).

### 📝 Linter (`lint.yaml`)

**Goal**: Act as a quality gate ensuring the codebase remains tidy, readable, and free of common coding errors.

- **Job 1: Frontend**:
  - `yarn typecheck`: Runs `svelte-check` to verify TypeScript types and Svelte 5 component props.
  - `yarn check-format`: Runs **Prettier** to check that indentation, line breaks, and quotation marks follow repository style.
- **Job 2: Backend Formatting**:
  - Runs `cargo fmt --all --check` to verify that all Rust code follows standard Rust formatting conventions.
- **Job 3: Backend Linter**:
  - Runs **Clippy** (`cargo clippy`), Rust's official code analysis tool that flags performance issues, dead code, or unidiomatic patterns.

### 🧪 Tests (`test.yaml`)

**Goal**: Ensure that existing features keep working as expected when new code is added.

- **Key Steps**:
  1. Installs the project dependencies with `yarn install --frozen-lockfile`.
  2. Runs the test suite via `yarn test` (powered by **Vitest**).
  3. Tests UI logic, utility helpers, and state stores to prevent functional regressions.

### 🏭 Create Release (`release.yaml`)

**Goal**: Automate the entire process of publishing a new public version of the OpenGOAL Launcher with zero manual copy-pasting.

- **Trigger**: Manually started by maintainers via GitHub's interface (`workflow_dispatch`), choosing between a `patch` (bug fix), `minor` (new feature), or `major` (breaking redesign) version bump.
- **Pipeline Progression**:
  1. **`create-tag`**: Bumps the version number in `package.json`, `Cargo.toml`, and `tauri.conf.json`, generates release notes metadata, commits the changes with `OpenGOALBot`, and creates a Git tag (e.g., `v2.11.2`).
  2. **`create-release`**: Uses GitHub CLI (`gh release create`) to create a draft release on GitHub with an automated changelog.
  3. **`build-assets`**: Compiles native production installers on Windows, Linux, and macOS, and uploads them to the GitHub release draft.
  4. **`update-release-manifests`**: Generates and updates `latest-release.json` files so that already-installed launchers know a new update is available and can download it automatically.
  5. **`publish-release`**: Publishes the draft release, making it visible to all players worldwide.

---

## 4. Summary Table

```
Code Change (Push / PR)
   ├── 📝 Linter (Checks formatting & TypeScript types)
   ├── 🧪 Tests (Runs Vitest test suite)
   └── 🔨 Build (Compiles app on Windows, Linux, and macOS)

Maintainer Triggers Release
   └── 🏭 Create Release
         ├── 1. Bump version & tag Git commit
         ├── 2. Create draft GitHub release
         ├── 3. Build installers for all platforms
         ├── 4. Update auto-updater JSON manifests
         └── 5. Publish release to the public
```

---

# 🇫🇷 Version Française

## 1. Introduction : Qu'est-ce que la CI/CD et les GitHub Actions ?

Lorsque plusieurs développeurs collaborent sur une application, vérifier manuellement que le code compile, qu'il fonctionne et qu'il respecte les normes de présentation sur chaque ordinateur est long et source d'erreurs.

**GitHub Actions** est un outil d'automatisation directement intégré à GitHub. On peut l'imaginer comme une **chaîne d'assemblage automatisée** :

- Dès qu'un développeur propose une modification (via un `push` ou une `Pull Request`), GitHub démarre des machines virtuelles dans le cloud (Linux, Windows, macOS).
- Il exécute automatiquement des vérifications (**CI** - _Intégration Continue_) : tests, respect des styles de code, compilation.
- Au moment d'une mise à jour publique, il fabrique et distribue automatiquement les installateurs pour les joueurs (**CD** - _Déploiement Continu_).

Dans ce projet, ces tâches automatisées (appelées _workflows_) sont définies dans des fichiers de configuration situés dans le dossier `.github/workflows/`.

---

## 2. Vue d'ensemble des 4 Workflows

| Fichier Workflow | Nom               | Objectif                                                   | Quand se déclenche-t-il ?                     |
| :--------------- | :---------------- | :--------------------------------------------------------- | :-------------------------------------------- |
| `build.yaml`     | 🔨 Build          | Compile l'application sur Windows, Linux et macOS          | À chaque `push`/PR sur `main` et à chaque tag |
| `lint.yaml`      | 📝 Linter         | Vérifie la syntaxe, la présentation et le typage           | À chaque `push` et PR sur `main`              |
| `test.yaml`      | 🧪 Tests          | Lance les tests unitaires et fonctionnels                  | À chaque `push`/PR sur `main` et à chaque tag |
| `release.yaml`   | 🏭 Create Release | Fabrique les installateurs finaux et publie la mise à jour | Déclenché manuellement par un mainteneur      |

---

## 3. Détail de chaque Workflow

### 🔨 Build (`build.yaml`)

**Rôle** : S'assurer que le projet compile correctement et sans aucune erreur sur tous les systèmes d'exploitation pris en charge.

- **Matrice multi-systèmes** : S'exécute en parallèle sur 4 machines virtuelles :
  1. `ubuntu-24.04` (Linux)
  2. `windows-latest` (Windows)
  3. `macos-15` (Mac avec puce Apple Silicon / ARM)
  4. `macos-15-intel` (Mac avec processeur Intel)
- **Étapes principales** :
  1. Installe les environnements de travail **Node.js** (v24) et **Rust**.
  2. Réutilise les fichiers de compilation précédents mis en cache pour accélérer le processus (`rust-cache`).
  3. Sur Linux, installe les bibliothèques graphiques indispensables (`libwebkit2gtk`, `libxdo`, etc.).
  4. Installe les dépendances web (`yarn install`) et compile l'interface (`yarn build`).
  5. Exécute Tauri pour assembler le binaire natif et créer les fichiers distribuables (comme le paquet `.AppImage` sous Linux).

### 📝 Linter (`lint.yaml`)

**Rôle** : Servir de « contrôle qualité » pour garantir un code propre, lisible et exempt de mauvaises pratiques.

- **Tâche 1 : Frontend (Interface)** :
  - `yarn typecheck` : Vérifie la cohérence des types TypeScript et des composants Svelte 5 via `svelte-check`.
  - `yarn check-format` : Vérifie avec **Prettier** que l'indentation, les espacements et les guillemets respectent scrupuleusement la charte du projet.
- **Tâche 2 : Formatage Backend (Rust)** :
  - `cargo fmt --all --check` : Vérifie que le code Rust applique les règles officielles de formatage.
- **Tâche 3 : Analyseur Statique Rust** :
  - Exécute **Clippy** (`cargo clippy`), l'outil officiel d'inspection de code Rust qui détecte d'éventuels oublis, portions de code inutiles ou maladresses de programmation.

### 🧪 Tests (`test.yaml`)

**Rôle** : Vérifier que les fonctionnalités existantes continuent de fonctionner sans régression lors de l'ajout de nouveau code.

- **Étapes principales** :
  1. Installe les paquets nécessaires avec `yarn install --frozen-lockfile`.
  2. Exécute la suite de tests automatisés via `yarn test` (propulsée par **Vitest**).
  3. Teste la logique des composants, la gestion d'état et les fonctions utilitaires.

### 🏭 Création de Version / Release (`release.yaml`)

**Rôle** : Automatiser de A à Z la publication d'une nouvelle version officielle du launcher à destination des joueurs, sans manipulation manuelle risquée.

- **Déclenchement** : Lancé manuellement par l'équipe via l'onglet _Actions_ de GitHub, en sélectionnant le type d'incrément de version :
  - `patch` (correction de bugs légers, ex : `2.11.1` -> `2.11.2`)
  - `minor` (nouvelles fonctionnalités, ex : `2.11.1` -> `2.12.0`)
  - `major` (refonte majeure)
- **Déroulement de la publication** :
  1. **`create-tag`** : Met à jour automatiquement le numéro de version dans `package.json`, `Cargo.toml` et `tauri.conf.json`, génère les métadonnées de nouveautés, effectue un commit automatique au nom de `OpenGOALBot` et crée le tag Git (ex : `v2.11.2`).
  2. **`create-release`** : Crée un brouillon de version sur GitHub avec la liste automatique des changements récents.
  3. **`build-assets`** : Compile les installateurs finaux pour Windows, Linux et macOS et les téléverse sur le brouillon de release.
  4. **`update-release-manifests`** : Met à jour les fichiers JSON d'information (`latest-release.json`) qui permettent aux launchers déjà installés chez les utilisateurs de détecter la mise à jour et de se mettre à niveau automatiquement.
  5. **`publish-release`** : Rend la publication publique pour l'ensemble de la communauté.

---

## 4. Tableau Récapitulatif

```
Modification de Code (Push / Pull Request)
   ├── 📝 Linter (Contrôle du style et typage TypeScript)
   ├── 🧪 Tests (Exécution des tests automatisés Vitest)
   └── 🔨 Build (Compilation sur Windows, Linux et macOS)

Publication d'une Nouvelle Version (Action Manuelle)
   └── 🏭 Create Release
         ├── 1. Incrémentation de version et création du tag
         ├── 2. Création du brouillon de publication GitHub
         ├── 3. Compilation des installateurs pour tous les OS
         ├── 4. Mise à jour des manifestes d'auto-update
         └── 5. Mise en ligne publique de la mise à jour
```
