+++
title = "Installation"
description = "Installeur ou version portable : comment installer Misty, où il s'installe et comment le désinstaller."
section = "start"
+++

Misty existe sous deux formes, disponibles sur la [page de téléchargement](/#download) et dans les [releases GitHub](https://github.com/Misty-MC/Misty/releases). Les deux se mettent à jour toutes seules.

## Installeur (recommandé)

1. Télécharge `Misty-Setup-<version>.exe`.
2. Lance-le et suis les étapes. Tu peux choisir d'ajouter un raccourci sur le bureau.
3. Démarre Misty depuis le menu Démarrer.

Ce que fait l'installeur :

- il installe Misty dans `%LOCALAPPDATA%\Programs\Misty`, **sans droits administrateur** ;
- il ajoute un raccourci dans le menu Démarrer (et sur le bureau si tu l'as demandé) ;
- il inscrit Misty dans les **Applications installées** de Windows, pour une désinstallation propre.

## Version portable

1. Télécharge `Misty-<version>.zip`.
2. Extrais-le où tu veux, par exemple sur une clé USB.
3. Lance `Misty-<version>\Misty.exe`.

Rien n'est installé dans Windows. Les données du jeu vont quand même dans `%LOCALAPPDATA%\Misty` sur le PC où tu joues. Voir [Fichiers et dossiers](/docs/files-and-folders).

> [!TIP]
> Les deux versions utilisent le même dossier de données : tu peux passer de l'une à l'autre sans perdre tes profils, tes mondes ni tes réglages.

## Rien d'autre à installer

`Misty.exe` contient tout ce dont il a besoin : polices, traductions et images. Java est téléchargé automatiquement quand une version de Minecraft en a besoin.

## Désinstaller

- **Installeur :** désinstalle Misty depuis **Paramètres → Applications → Applications installées** dans Windows. Tes données de jeu (versions, profils, mondes) sont gardées, sauf si tu demandes leur suppression.
- **Portable :** supprime le dossier extrait.

Pour effacer aussi **toutes** les données de jeu, utilise **Tout supprimer** dans la carte Stockage des [Paramètres](/docs/settings) avant de désinstaller, ou supprime toi-même `%LOCALAPPDATA%\Misty`.

> [!WARNING]
> Supprimer le dossier de données supprime aussi les mondes de tous les profils. Sauvegarde d'abord les dossiers `saves` que tu veux garder.
