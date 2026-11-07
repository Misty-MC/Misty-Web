+++
title = "Fichiers et dossiers"
description = "Où Misty range le jeu, tes profils, tes comptes et ses réglages."
section = "help"
+++

## Le dossier de données

Tout ce que Misty télécharge ou crée va dans **`%LOCALAPPDATA%\Misty`**. Colle ce chemin dans la barre d'adresse de l'Explorateur Windows pour l'ouvrir, ou utilise **Ouvrir le dossier du jeu** dans la barre latérale.

| Fichier / dossier | Contenu |
|---|---|
| `data.txt` | Réglages : dernier compte et dernière version, RAM, langue, versions affichées, Rich Presence, dernier profil… |
| `listpseudo.txt` | Comptes hors-ligne |
| `msa.json` | Comptes Microsoft connectés |
| `instances\<id>\` | Un dossier par [profil](/docs/profiles) |
| `versions\`, `libraries\`, `assets\`, Java | L'installation de Minecraft, partagée par tous les profils |

## Le dossier d'un profil

Chaque profil de `instances\<id>\` contient :

| Fichier / dossier | Contenu |
|---|---|
| `instance.json` | Les réglages du profil : nom, version, mod loader… |
| `mods\` | Mods |
| `resourcepacks\` | Packs de ressources |
| `shaderpacks\` | Shaders |
| `saves\` | Mondes |
| `misty-contenu.json` | Ce que Misty sait du contenu installé |

**Ouvrir le dossier du profil**, sur la page du profil, l'ouvre directement.

## Les journaux du jeu

Minecraft écrit ses journaux dans le dossier de jeu utilisé (celui du profil, ou le dossier de données en *Jeu rapide*) :

- `logs\latest.log` : le journal de la dernière partie ;
- `crash-reports\` : un rapport par plantage.

Joins ces fichiers quand tu [signales un problème](/docs/troubleshooting#signaler-un-bug).

## Le dossier de l'application

- Installeur : `%LOCALAPPDATA%\Programs\Misty\Misty-<version>\Misty.exe`
- Portable : là où tu as extrait le zip.

> [!TIP]
> Pour donner ta propre icône à Misty, place un fichier `Icon.ico` à côté de `Misty.exe`.
