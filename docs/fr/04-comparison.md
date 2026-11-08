+++
title = "Misty face aux autres launchers"
description = "Misty comparé à Lunar Client, Badlion Client, Horizon, Modrinth App et l'app CurseForge : fonctionnalités, comptes, mods et performances mesurées."
section = "start"
+++

Il existe beaucoup de façons de lancer Minecraft : Java Edition. Elles ne font pas toutes le même travail : voici une comparaison honnête pour t'aider à choisir. Misty ne cherche pas à remplacer un client PvP : c'est un **launcher général et léger**, pour toutes les versions et tous les mod loaders.

> [!NOTE]
> Cette page compare ce que propose chaque launcher. Les fonctionnalités évoluent : si une information n'est plus à jour, dis-le-nous sur [GitHub](https://github.com/Misty-MC/Misty/issues).

## En un coup d'œil

| | Misty | Lunar Client | Badlion Client | Horizon | Modrinth App | App CurseForge |
|---|---|---|---|---|---|---|
| **Type** | Launcher général | Client PvP | Client PvP | Réécriture du jeu en Rust + launcher | Launcher général | Launcher général |
| **Prix** | Gratuit | Gratuit (cosmétiques payants) | Gratuit (cosmétiques payants) | Gratuit | Gratuit (abonnement facultatif) | Gratuit |
| **Open source** | Oui (MIT) | Non | Non | Non vérifié | Oui (GPL-3.0) | Non |
| **Technologie** | Rust + Slint (natif) | Electron | Electron | Rust | Tauri (Rust + interface web) | Electron |
| **Toutes les versions de Minecraft** | Oui, jusqu'aux alphas et à l'indev | Une sélection de versions | Une sélection de versions | Ses propres versions | Oui | Oui |
| **Forge, NeoForge, Fabric, Quilt** | Oui, plus OptiFine et LiteLoader | Non (mods intégrés) | Non (mods intégrés) | Non vérifié | Oui | Oui |
| **Mods de Modrinth** | Oui | Non | Non | Oui | Oui | Non |
| **Mods de CurseForge** | Oui | Non | Non | Non vérifié | Non | Oui |
| **Modpacks** | Modrinth (.mrpack) et CurseForge | Non | Non | Import | Modrinth | CurseForge |
| **Compte Microsoft** | Oui, plusieurs | Oui | Oui | Oui | Oui | Oui |
| **Comptes hors-ligne** | Oui | Non | Non | Oui | Non | Non |
| **Éditeur de packs de textures** | Oui | Non | Non | Non | Non | Non |
| **Cosmétiques** | Bientôt (cape et élytres Misty à essayer) | Oui | Oui | Non | Non | Non |
| **Windows / macOS / Linux** | Windows | Les trois | Windows, macOS | Non vérifié | Les trois | Les trois |

**En résumé :**

- Tu joues en **PvP sur quelques versions** et tu veux des mods FPS intégrés : Lunar Client ou Badlion Client.
- Tu veux **toutes les versions, tous les loaders et les mods de Modrinth comme de CurseForge**, dans une petite app native avec des comptes hors-ligne : **Misty**.
- Tu n'utilises que **Modrinth** : la Modrinth App est excellente. Seulement des modpacks **CurseForge** : l'app CurseForge.
- Tu es curieux d'un **autre moteur** pour le jeu lui-même : Horizon réécrit Minecraft en Rust ([playhorizon.dev](https://playhorizon.dev)).

## Performances

Mesuré sur un **Ryzen 7 7435HS, 24 Go de RAM, Windows 10**, Misty 0.3.4 (version release), sur l'accueil, 15 secondes après le lancement. Moyenne de 3 lancements.

| | Misty |
|---|---|
| **Taille du téléchargement** | 38 Mo (un seul `Misty.exe`) |
| **Fenêtre affichée après** | environ 50 ms (l'écran de chargement, puis l'accueil) |
| **Mémoire au repos (plage de travail)** | environ 120 Mo |
| **Mémoire au repos (privée)** | environ 170 Mo |

> [!IMPORTANT]
> On ne publie que des chiffres mesurés par nous-mêmes. Les autres launchers n'ont pas encore été mesurés sur cette machine, ils ne sont donc pas dans ce tableau : on préfère une case vide à un chiffre inventé.

### Mesure toi-même

La méthode est simple et marche pour n'importe quel launcher :

1. Ferme tous les launchers et redémarre, pour que rien ne soit déjà en mémoire.
2. Lance le launcher avec un chronomètre, et note quand sa fenêtre apparaît.
3. Attends 15 secondes sur son accueil sans rien toucher.
4. Dans le **Gestionnaire des tâches** (onglet Détails), note la **plage de travail** et la **mémoire privée** de ses processus (une app Electron en lance plusieurs : additionne-les).
5. Recommence trois fois et fais la moyenne.

La mémoire utilisée par **le jeu lui-même** dépend de Minecraft, de tes mods et de la RAM que tu lui donnes ([Options](/docs/settings)), pas du launcher : c'est pour ça qu'elle n'est pas comparée ici.
