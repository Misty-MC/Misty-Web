+++
title = "Éditeur de textures"
description = "Crée un pack de ressources en modifiant les textures de Minecraft pixel par pixel, puis exporte-le en .zip."
section = "tools"
+++

> [!WARNING]
> L'éditeur de packs de textures est **expérimental** : des bugs peuvent apparaître.

Ouvre **Packs de textures** dans la barre latérale pour voir tes packs, les éditer, les exporter ou les supprimer.

## Créer un pack

1. Clique sur **Créer un pack**.
2. Donne-lui un **nom** et une **description**.
3. Choisis la **version de Minecraft** visée. Les packs de ressources existent depuis la 1.6.1.
4. Clique sur **Créer le pack**.

Les textures d'origine de cette version sont téléchargées **une seule fois**, puis réutilisées pour tous les packs de la même version.

## Modifier les textures

L'éditeur montre toutes les textures du jeu, rangées par catégories (blocs de construction, redstone, entités, interface…) avec un compteur. Utilise le champ de recherche pour en trouver une par son nom.

- **Toutes les textures** : toutes les textures de la version.
- **Mes textures** : seulement celles que tu as modifiées.

Clique sur une texture pour l'ouvrir, puis dessine.

### Outils et raccourcis

| Outil | Raccourci |
|---|---|
| Crayon | `B` |
| Gomme | `E` |
| Pot de peinture | `G` |
| Pipette | `I`, ou clic droit pour une pipette rapide |
| Annuler / Rétablir | `Ctrl+Z` / `Ctrl+Y` |
| Enregistrer | `Ctrl+S` |
| Zoom | `Ctrl` + molette |

Le panneau latéral affiche les **couleurs récentes** et les **couleurs de la texture**. Le bouton **Grille** affiche la grille des pixels. **Réinitialiser** remet la texture d'origine et retire ta version du pack.

La texture en cours est enregistrée quand tu quittes la page.

### Éditeur externe

Tu préfères Paint.NET ou un autre logiciel ? Clique sur **Éditeur externe** et choisis ton programme (par exemple `paintdotnet.exe` ; annule pour utiliser l'application par défaut). Modifie et enregistre dans ton logiciel : Misty recharge la texture automatiquement dès que le fichier change.

## Icône du pack

Dans l'éditeur, **Changer l'icône du pack** définit l'image affichée dans la liste des packs de ressources de Minecraft.

## Exporter et utiliser le pack

1. Clique sur **Exporter** et choisis où enregistrer le `.zip`.
2. Place le zip dans le dossier `resourcepacks` d'un [profil](/docs/profiles).
3. Dans Minecraft, active-le dans **Options → Packs de ressources**.

Misty écrit le bon `pack_format` pour la version, y compris pour les versions très anciennes et très récentes.
