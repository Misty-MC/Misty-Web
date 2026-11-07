+++
title = "Mods, packs et shaders"
description = "Trouve du contenu sur Modrinth ou CurseForge, installe-le avec ses dépendances et gère ce que contient un profil."
section = "play"
+++

Le contenu s'installe par [profil](/docs/profiles). Ouvre un profil, puis utilise ses deux onglets : **Découvrir** pour trouver du contenu, **Contenu** pour gérer ce qui est installé.

## Découvrir

1. Choisis la plateforme : **Modrinth** ou **CurseForge**.
2. Choisis le type : **Mods**, **Packs de ressources**, **Datapacks** ou **Shaders**.
3. Cherche, et trie par pertinence, téléchargements, popularité, plus récents, mis à jour, en vedette ou nom.
4. Clique sur **Installer**.

Les résultats sont filtrés pour rester **compatibles** avec la version et le mod loader du profil. Quand un mod a besoin d'autres mods, Misty installe aussi ses **dépendances** et t'indique combien.

> [!NOTE]
> Les mods ont besoin d'un mod loader. Sur un profil Vanilla ou OptiFine, Misty te demande d'abord de choisir un loader. Voir [Versions et mod loaders](/docs/versions-and-loaders).

### Datapacks

Les datapacks s'installent **dans un monde**. Misty te demande lequel utiliser. Si le profil n'a encore aucun monde, lance-le une fois, crée un monde, puis reviens.

### Shaders

Les shaders ont besoin d'OptiFine, ou du mod Iris (Fabric, NeoForge) ou Oculus (Forge). S'il manque, Misty propose de l'installer avec le shader. Voir [Shaders](/docs/versions-and-loaders#shaders).

## Contenu

L'onglet **Contenu** liste tout ce qui est installé dans le profil, avec un filtre par type et un champ de recherche. Pour chaque élément, tu peux :

- l'**activer ou le désactiver** : le fichier est renommé avec l'extension `.disabled` et le jeu l'ignore ;
- le **supprimer**.

Les fichiers que tu as ajoutés à la main dans les dossiers sont aussi identifiés, grâce à leur hash sur Modrinth ou leur empreinte sur CurseForge : ils s'affichent avec leur vrai nom.

## CurseForge

Certains auteurs CurseForge n'autorisent pas le téléchargement depuis d'autres applications que CurseForge. Dans ce cas, Misty liste les fichiers et ouvre leurs pages : télécharge-les et place-les dans le dossier indiqué. Voir [Modpacks](/docs/modpacks#téléchargements-manuels).
