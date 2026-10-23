> **Language / Langue :** [🇬🇧 English Version](#-english-version) &nbsp;•&nbsp; [🇫🇷 Version Française](#-version-française)

## Summary / Sommaire

- [🇬🇧 English Version](#-english-version)
  - [1. Introduction: The Big Picture](#1-introduction-the-big-picture)
  - [2. Why this Architecture? (The "Best of Both Worlds")](#2-why-this-architecture-the-best-of-both-worlds)
  - [3. The Key Technologies](#3-the-key-technologies)
    - [🦀 The Backend: Rust & Tauri v2](#-the-backend-rust--tauri-v2)
    - [⚡ The Frontend: Svelte 5 & TypeScript](#-the-frontend-svelte-5--typescript)
    - [🎨 Styling: Tailwind CSS v4 & Flowbite Svelte](#-styling-tailwind-css-v4--flowbite-svelte)
    - [📦 Build & Bundling: Vite](#-build--bundling-vite)
  - [4. How the Pieces Talk to Each Other](#4-how-the-pieces-talk-to-each-other)
  - [5. The OpenGOAL Native Binaries (`gk`, `extractor`, `goalc`)](#5-the-opengoal-native-binaries-gk-extractor-goalc)
  - [6. Architecture Flowchart](#6-architecture-flowchart)
- [🇫🇷 Version Française](#-version-française)
  - [1. Introduction : La Vue d'Ensemble](#1-introduction--la-vue-densemble)
  - [2. Pourquoi cette Architecture ? (Le "Meilleur des Deux Mondes")](#2-pourquoi-cette-architecture--le-meilleur-des-deux-mondes)
  - [3. Les Technologies Clés](#3-les-technologies-clés)
    - [🦀 Le Backend : Rust & Tauri v2](#-le-backend--rust--tauri-v2)
    - [⚡ Le Frontend : Svelte 5 & TypeScript](#-le-frontend--svelte-5--typescript)
    - [🎨 L'Habillage Visuel : Tailwind CSS v4 & Flowbite Svelte](#-lhabillage-visuel--tailwind-css-v4--flowbite-svelte)
    - [📦 L'Outillage de Compilation : Vite](#-loutillage-de-compilation--vite)
  - [4. Comment les Composants Communiquent-ils ?](#4-comment-les-composants-communiquent-ils-)
  - [5. Les Outils Natifs d'OpenGOAL (`gk`, `extractor`, `goalc`)](#5-les-outils-natifs-dopengoal-gk-extractor-goalc)
  - [6. Schéma Récapitulatif](#6-schéma-récapitulatif)

---

# 🇬🇧 English Version

## 1. Introduction: The Big Picture

The **OpenGOAL Launcher** looks like a smooth, modern application where you can click "Play", install mods, change settings, and download texture packs with ease.

Behind this user-friendly interface is a well-coordinated team of different software technologies. Each technology was selected for a specific purpose: some handle raw computing power and file operations on your computer, while others create the visual interface you interact with.

---

## 2. Why this Architecture? (The "Best of Both Worlds")

When building desktop applications, developers usually face a dilemma:

1. **Pure Native Apps (C++ or C#)**: Very fast and lightweight, but designing modern, responsive, multi-language user interfaces with smooth animations in traditional native frameworks is complex and slow to build.
2. **Traditional Web-based Desktop Apps (Electron)**: Fast to build using HTML and CSS, but they bundle a complete Google Chrome browser and Node.js runtime with them, leading to large file sizes and heavy RAM usage.

**The OpenGOAL Launcher chooses a smarter third way: [Tauri v2](https://tauri.app/)**.

- It uses the operating system's built-in web renderer (WebView2 on Windows, WebKit on Linux/macOS) to display an interface built with **HTML/CSS/Svelte**.
- It uses **Rust** for everything running under the hood (accessing files, starting games, downloading files).
- **Result**: The launcher opens instantly, consumes only around 30 to 50 MB of RAM, and has a sleek, responsive interface.

---

## 3. The Key Technologies

### 🦀 The Backend: Rust & Tauri v2

- **What is it?** Rust is a modern programming language known for extreme speed, rock-solid stability, and memory safety without needing a garbage collector.
- **Why is it used?** Because web browsers are normally locked inside a security "sandbox" and cannot freely launch `.exe` files, read your hard drive, or manipulate ISO files. Rust operates outside the sandbox with full system permissions.
- **What does it do in the launcher?**
  - Reads and saves configuration files (`settings.json`).
  - Downloads mod ZIP files and official OpenGOAL toolkits from GitHub.
  - Detects save games and monitors play time.
  - Launches the game engine executable (`gk.exe`) with the correct command-line flags.

### ⚡ The Frontend: Svelte 5 & TypeScript

- **What is it?** Svelte is a modern UI framework. Unlike other web frameworks (like React) that do heavy work inside the user's browser, Svelte transforms components into ultra-lean, pure JavaScript code ahead of time during the build process.
- **Why is it used?** It is simple to read, incredibly fast, and Svelte 5 introduces "Runes" (`$state`, `$derived`, `$props`, `$effect`) that make keeping the screen synchronized with data virtually effortless.
- **What is TypeScript?** TypeScript adds type checking on top of JavaScript. It ensures variables hold the right kind of information (e.g. preventing a number from accidentally being treated as a piece of text), catching bugs before the app ever runs.

### 🎨 Styling: Tailwind CSS v4 & Flowbite Svelte

- **What is it?** Tailwind CSS allows styling components directly in HTML using intuitive utility classes (like `flex`, `gap-4`, `rounded-full`). Flowbite Svelte provides pre-built, accessible building blocks such as dropdown menus, tooltips, toggle switches, and progress bars.
- **Why is it used?** It avoids writing thousands of lines of custom CSS from scratch while guaranteeing visual consistency, smooth animations, and dark mode support.

### 📦 Build & Bundling: Vite

- **What is it?** Vite is the frontend build tool.
- **Why is it used?** During development, Vite provides **Hot Module Replacement (HMR)**: whenever you edit a Svelte file, the change appears instantly in the running app without reloading the whole launcher. When packaging the app, Vite bundles and minifies the code for production.

---

## 4. How the Pieces Talk to Each Other

The Svelte frontend (visual layer) and the Rust backend (system engine) live in separate spaces. They communicate using two simple mechanisms provided by Tauri:

```
[ Frontend (Svelte) ]  ────── (1) Tauri Invoke (RPC) ──────>  [ Backend (Rust) ]
                       <───── (Returns result or error) ────

[ Frontend (Svelte) ]  <───── (2) Tauri Events (Stream) ────  [ Backend (Rust) ]
```

1. **Commands (Tauri Invoke / RPC)**:
   - _Analogy_: Asking a waiter at a restaurant for food.
   - _Example_: The user clicks "Play". The Svelte button calls `launchGame()`. This sends an IPC message to Rust: _"Please launch Jak 2"_. Rust executes the command, launches `gk.exe`, and replies: _"All good!"_.
2. **Events (Tauri Listen / Emit)**:
   - _Analogy_: A progress loudspeaker.
   - _Example_: While extracting game assets from an ISO, Rust repeatedly broadcasts: _"Progress is at 45%"_, _"Progress is at 50%"_. The frontend listens for this event and moves the progress bar smoothly.

---

## 5. The OpenGOAL Native Binaries (`gk`, `extractor`, `goalc`)

The launcher doesn't just display menus; it coordinates three native OpenGOAL programs:

1. **`extractor`**: The extraction tool. When you feed it a PlayStation 2 ISO disc image of _Jak and Daxter_, it extracts the 3D models, textures, animations, level files, and audio tracks onto your computer.
2. **`goalc`**: The GOAL compiler. OpenGOAL games were originally written in a proprietary programming language created by Naughty Dog called GOAL. `goalc` compiles game source code into machine code that your PC can run natively.
3. **`gk` (Game Kernel)**: The heart of the project. It is the native PC port of the game engine (written in C++ and GOAL). It creates the game window, plays the audio, processes your controller inputs, reads your saves, and renders the graphics at high resolutions and 60+ FPS.

---

## 6. Architecture Flowchart

```
┌─────────────────────────────────────────────────────────────┐
│                      USER INTERFACE                         │
│   Svelte 5 + TypeScript + Tailwind CSS + Flowbite Svelte    │
│   (Game library, Mod list, Buttons, Settings, UI toggles)   │
└──────────────────────────────┬──────────────────────────────┘
                               │  Tauri IPC (Commands & Events)
┌──────────────────────────────▼──────────────────────────────┐
│                     BACKEND RUNTIME                         │
│                    Rust & Tauri v2                          │
│   - Config file manager (settings.json)                     │
│   - GitHub downloader (mods, texture packs, tools)          │
│   - System process manager                                  │
└───────────────┬──────────────────────────────┬──────────────┘
                │ Starts subprocesses          │ Reads & writes
┌───────────────▼──────────────┐ ┌─────────────▼──────────────┐
│      OPENGOAL BINARIES       │ │        LOCAL STORAGE       │
│ - extractor (ISO extraction) │ │ - settings.json (Config)   │
│ - goalc (Compiler)           │ │ - %APPDATA%/OpenGOAL/saves │
│ - gk (Game Engine executable)│ │ - Extracted game assets    │
└──────────────────────────────┘ └────────────────────────────┘
```

---

# 🇫🇷 Version Française

## 1. Introduction : La Vue d'Ensemble

L'**OpenGOAL Launcher** se présente sous la forme d'une interface claire et fluide qui permet en quelques clics de lancer les jeux de la trilogie _Jak and Daxter_, d'installer des mods, de personnaliser des packs de textures et de modifier ses réglages.

Derrière cette apparente simplicité se cache un assemblage harmonieux de plusieurs technologies complémentaires. Chacune a été choisie pour une raison précise : certaines s'occupent de la puissance brute et des fichiers de votre machine, tandis que d'autres gèrent l'aspect visuel avec lequel vous interagissez.

---

## 2. Pourquoi cette Architecture ? (Le "Meilleur des Deux Mondes")

Lorsqu'on crée un logiciel de bureau, on se retrouve souvent face à un dilemme :

1. **Les applications 100% natives (C++ ou C#)** : Très rapides et légères, mais concevoir une interface moderne, esthétique, animée et disponible en plusieurs langues avec les outils graphiques natifs traditionnels est très long et complexe.
2. **Les applications web traditionnelles pour bureau (comme Electron)** : Très rapides à concevoir grâce aux technologies du web (HTML/CSS), mais elles embarquent une copie complète du navigateur Google Chrome et de Node.js, ce qui les rend lourdes sur le disque et gourmandes en mémoire vive (RAM).

**L'OpenGOAL Launcher utilise une troisième solution bien plus élégante : [Tauri v2](https://tauri.app/)**.

- Il utilise le moteur d'affichage web déjà présent sur votre système d'exploitation (WebView2 sous Windows, WebKit sous Linux/macOS) pour afficher l'interface conçue en **HTML/CSS/Svelte**.
- Il utilise **Rust** pour tout ce qui tourne sous le capot (gestion des fichiers, téléchargements, exécution des jeux).
- **Résultat** : Le launcher s'ouvre instantanément, ne consomme que 30 à 50 Mo de RAM et offre une interface moderne et réactive.

---

## 3. Les Technologies Clés

### 🦀 Le Backend : Rust & Tauri v2

- **Qu'est-ce que c'est ?** Rust est un langage de programmation moderne réputé pour sa rapidité extrême, sa stabilité et sa sécurité de gestion de mémoire.
- **Pourquoi l'utilise-t-on ?** Les navigateurs web sont isolés dans un « bac à sable » sécurisé : ils ne peuvent pas ouvrir vos dossiers personnels, manipuler des ISOs ou lancer des fichiers `.exe` sur votre ordinateur. Rust opère en dehors de ce bac à sable avec tous les accès système nécessaires.
- **Que fait-il concrètement dans le launcher ?**
  - Il lit et enregistre les réglages de configuration (`settings.json`).
  - Il télécharge les archives ZIP des mods et les outils officiels depuis GitHub.
  - Il détecte les sauvegardes et chronomètre le temps de jeu.
  - Il lance le moteur de jeu (`gk.exe`) avec les bons arguments en ligne de commande.

### ⚡ Le Frontend : Svelte 5 & TypeScript

- **Qu'est-ce que c'est ?** Svelte est un outil de création d'interfaces graphiques. Contrairement à d'autres solutions lourdes (comme React) qui effectuent de nombreux calculs dans le navigateur, Svelte transforme les composants en code JavaScript ultra-léger et direct dès la phase de compilation.
- **Pourquoi l'utilise-t-on ?** Le code est simple, élégant et très performant. Svelte 5 introduit les « Runes » (`$state`, `$derived`, `$props`, `$effect`) qui permettent de synchroniser instantanément l'affichage dès qu'une donnée change, sans effort.
- **Et TypeScript ?** TypeScript est une surcouche à JavaScript qui vérifie la nature des données manipulées (le « typage »). Il empêche par exemple d'utiliser par erreur du texte à la place d'un nombre, éliminant les bugs avant même le lancement de l'application.

### 🎨 L'Habillage Visuel : Tailwind CSS v4 & Flowbite Svelte

- **Qu'est-ce que c'est ?** Tailwind CSS permet de styliser les éléments directement dans le code HTML à l'aide de classes intuitives (ex : `flex` pour aligner, `gap-4` pour espacer, `rounded-full` pour arrondir). Flowbite Svelte fournit des éléments interactifs prêts à l'emploi (boutons, menus déroulants, infobulles, commutateurs switch, barres de progression).
- **Pourquoi l'utilise-t-on ?** Cela évite de réinventer la roue et d'écrire des milliers de lignes de CSS maison, tout en garantissant un thème sombre/clair impeccable et des animations fluides.

### 📦 L'Outillage de Compilation : Vite

- **Qu'est-ce que c'est ?** Vite est l'outil d'assemblage et serveur de développement.
- **Pourquoi l'utilise-t-on ?** En phase de développement, Vite propose le **Hot Module Replacement (HMR)** : dès que vous modifiez un fichier Svelte, le résultat s'affiche instantanément dans le launcher sans devoir le redémarrer. Pour la version finale, il compresse et optimise l'ensemble des fichiers web.

---

## 4. Comment les Composants Communiquent-ils ?

L'interface graphique Svelte (ce que vous voyez) et le moteur Rust (ce qui agit sur l'ordinateur) fonctionnent dans deux espaces distincts. Ils s'échangent des informations grâce à deux mécanismes prévus par Tauri :

```
[ Interface (Svelte) ]  ────── (1) Appel Tauri Invoke (RPC) ─────>  [ Moteur (Rust) ]
                        <───── (Réponse ou message d'erreur) ────

[ Interface (Svelte) ]  <───── (2) Événements Tauri (Flux direct) ─  [ Moteur (Rust) ]
```

1. **Les Commandes (Tauri Invoke / RPC)** :
   - _Analogie_ : Passer une commande à un serveur au restaurant.
   - _Exemple_ : Vous cliquez sur « Jouer ». Le bouton Svelte appelle `launchGame()`. Cela envoie un message à Rust : _« Peux-tu lancer Jak 2 ? »_. Rust exécute l'action, démarre `gk.exe` et répond : _« C'est fait ! »_.
2. **Les Événements (Tauri Listen / Emit)** :
   - _Analogie_ : Des annonces au micro en gare.
   - _Exemple_ : Lors de l'extraction des fichiers d'un jeu depuis une image ISO, Rust diffuse en direct : _« Progression à 45% »_, puis _« Progression à 50% »_. L'interface écoute ces messages et fait avancer la barre de chargement sans à-coups.

---

## 5. Les Outils Natifs d'OpenGOAL (`gk`, `extractor`, `goalc`)

Le launcher ne se contente pas d'afficher de jolis menus : il pilote trois programmes natifs essentiels du projet OpenGOAL :

1. **`extractor`** : L'outil d'extraction. Quand vous lui fournissez une image ISO officielle de votre jeu PS2 _Jak and Daxter_, il en extrait les textures, les modèles 3D, les niveaux et les musiques sur votre ordinateur.
2. **`goalc`** : Le compilateur GOAL. Les jeux de la trilogie ont été programmés à l'origine dans un langage sur-mesure inventé par le studio Naughty Dog appelé GOAL. `goalc` compile ce code source en langage machine compréhensible par votre processeur de PC.
3. **`gk` (Game Kernel)** : Le cœur du jeu. Il s'agit du moteur exécutable natif sur PC (écrit en C++ et GOAL). C'est lui qui ouvre la fenêtre de jeu, gère le son, lit votre manette, charge vos sauvegardes et affiche les graphismes en haute résolution à 60 images par seconde ou plus.

---

## 6. Schéma Récapitulatif

```
┌─────────────────────────────────────────────────────────────┐
│                    INTERFACE UTILISATEUR                    │
│   Svelte 5 + TypeScript + Tailwind CSS + Flowbite Svelte    │
│   (Bibliothèque de jeux, liste de mods, boutons, options)   │
└──────────────────────────────┬──────────────────────────────┘
                               │  Tauri IPC (Commandes & Événements)
┌──────────────────────────────▼──────────────────────────────┐
│                      MOTEUR APPLICATIF                      │
│                       Rust & Tauri v2                       │
│   - Gestionnaire de configuration (settings.json)           │
│   - Téléchargeur GitHub (mods, textures, mises à jour)      │
│   - Pilote des processus et fenêtres natives                │
└───────────────┬──────────────────────────────┬──────────────┘
                │ Démarre les sous-processus   │ Lit et écrit
┌───────────────▼──────────────┐ ┌─────────────▼──────────────┐
│     BINAIRES OPENGOAL        │ │       STOCKAGE LOCAL       │
│ - extractor (Extraction ISO) │ │ - settings.json (Options)  │
│ - goalc (Compilateur GOAL)   │ │ - %APPDATA%/OpenGOAL/saves │
│ - gk (Moteur de jeu PC)      │ │ - Données et assets du jeu │
└──────────────────────────────┘ └────────────────────────────┘
```
