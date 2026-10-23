# GitHub Workflows

This page explains the automated checks and the release process of the launcher. They run on GitHub's machines, so nobody has to repeat them by hand.

## Contents

- [1. What GitHub Actions does here](#1-what-github-actions-does-here)
- [2. The four workflows](#2-the-four-workflows)
- [3. Build](#3-build)
- [4. Linter](#4-linter)
- [5. Tests](#5-tests)
- [6. Create Release](#6-create-release)

## 1. What GitHub Actions does here

GitHub Actions runs scripts on fresh virtual machines hosted by GitHub, with Linux, Windows or macOS. Each script is called a workflow and is a YAML file in [.github/workflows/](../.github/workflows/). A workflow starts on an event: a push, a pull request, a new tag, or a click on **Run workflow** in the **Actions** tab of the repository.

The launcher uses workflows for two things:

- **Continuous integration (CI)**: every change is built, checked and tested, so a mistake shows up before it is merged.
- **Continuous delivery (CD)**: a maintainer publishes a new version in one click, and the workflow builds the installers for every system.

## 2. The four workflows

| Workflow       | File           | Starts on                                                                  | What it does                                                        |
| :------------- | :------------- | :------------------------------------------------------------------------- | :------------------------------------------------------------------ |
| Build          | `build.yaml`   | Pushes and pull requests on `main` and `master-dev`, `v*` tags, manual run | Builds the launcher on Linux, Windows and macOS.                    |
| Linter         | `lint.yaml`    | Pushes and pull requests on `main` and `master-dev`, manual run            | Checks the types and the formatting of the interface and Rust code. |
| Tests          | `test.yaml`    | Pushes and pull requests on `main` and `master-dev`, `v*` tags, manual run | Runs the interface tests.                                           |
| Create Release | `release.yaml` | Manual run only                                                            | Bumps the version, builds the installers and publishes the release. |

The jobs that build or check the launcher also download the submodules (`submodules: recursive`), because the Level Editor in `level-editor/` is compiled into the launcher.

## 3. Build

[build.yaml](../.github/workflows/build.yaml) checks that the launcher compiles into an installable application on each supported system. The same job runs four times in parallel, once per machine:

| Machine          | System                 |
| :--------------- | :--------------------- |
| `ubuntu-24.04`   | Linux                  |
| `windows-latest` | Windows                |
| `macos-15`       | macOS on Apple Silicon |
| `macos-15-intel` | macOS on Intel         |

On each machine:

1. Node.js 24 and stable Rust are installed, and the Rust build cache from earlier runs is restored to save time.
2. On Linux only, the system libraries Tauri needs (WebKitGTK, libxdo, libayatana-appindicator and others) are installed, with a Tauri CLI that supports the new AppImage format.
3. `yarn install --frozen-lockfile` installs the interface dependencies, then `yarn build` builds the interface.
4. Tauri builds the application.
5. The result is uploaded as a downloadable artifact of the run: an `.AppImage` on Linux, an `.msi` installer on Windows, an application bundle on macOS.

## 4. Linter

A linter reads code without running it and reports mistakes and style problems. [lint.yaml](../.github/workflows/lint.yaml) runs three jobs in parallel on Linux:

| Job                | Command                   | What it checks                                                                                          |
| :----------------- | :------------------------ | :------------------------------------------------------------------------------------------------------ |
| Frontend           | `yarn typecheck`          | TypeScript types and Svelte component properties, with `svelte-check`.                                  |
| Frontend           | `yarn check-format`       | The formatting of the interface files, with Prettier.                                                   |
| Backend Formatting | `cargo fmt --all --check` | The formatting of the Rust code in `src-tauri/`.                                                        |
| Backend Linter     | `cargo clippy`            | Common Rust mistakes and unidiomatic code, with Clippy. Reports only: its warnings do not fail the run. |

To fix a formatting failure on your computer, run `npx yarn format` for the interface, and `cargo fmt` in `src-tauri/` for the Rust code.

## 5. Tests

[test.yaml](../.github/workflows/test.yaml) runs the interface tests, written with Vitest in `src/**/*.test.ts`, so a change cannot silently break an existing behaviour:

1. `yarn install --frozen-lockfile` installs the dependencies.
2. `yarn test` runs the tests.

Run the same tests on your computer with `npx yarn test`. The Rust tests (`cargo test` in `src-tauri/`) are not run by this workflow.

## 6. Create Release

[release.yaml](../.github/workflows/release.yaml) publishes a new version of the launcher. A maintainer starts it from **Actions > Create Release > Run workflow** and picks the kind of version bump:

| Choice  | Use it for       | Example            |
| :------ | :--------------- | :----------------- |
| `patch` | Bug fixes        | `1.0.2` to `1.0.3` |
| `minor` | New features     | `1.0.2` to `1.1.0` |
| `major` | Breaking changes | `1.0.2` to `2.0.0` |

The workflow then runs four jobs, one after the other:

1. **Create New Tag** (`create-tag`):
   - Checks that `level-editor/` is at a tag of [whozghiar/jak-level-editor](https://github.com/whozghiar/jak-level-editor) whose release is published and contains `ogle-extract-windows-x64.zip` and `ogle-extract-linux-x64.zip`, the extractor that launcher users download. If not, the release stops here.
   - Writes the new version into `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`, and into the release history of the Linux app metadata (`resources/metadata/dev.opengoal.OpenGOAL.appdata.xml`).
   - Commits these changes in the name of the person who started the workflow, and tags the commit with the version, for example `v1.0.3`.
2. **Create Release** (`create-release`): creates a draft release on GitHub, with notes generated from the changes merged since the last release.
3. **Build** (`build-app`): builds the installers on the same four machines as the Build workflow and attaches them to the draft.
4. **Publish** (`publish-release`): `yarn update-release-meta` writes [.tauri/latest-release-v2.json](../.tauri/latest-release-v2.json), the file that installed launchers download to find out that an update exists, then publishes the release. The updated file is committed.
