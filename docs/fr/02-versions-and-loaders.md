+++
title = "Versions et mod loaders"
description = "Les versions de Minecraft affichées par Misty, et les mod loaders proposés pour chacune."
section = "play"
+++

## Versions de Minecraft

Misty liste les versions publiées par Mojang. Les **releases** sont toujours affichées. Les autres types s'activent dans la carte **Versions affichées** des [Paramètres](/docs/settings) :

- **Snapshots** : versions de test de la prochaine release ;
- **Bêtas** ;
- **Alphas et Indev** : les toutes premières versions du jeu.

Si la liste n'a pas pu être chargée (pas de connexion, par exemple), utilise le bouton **Recharger la liste des versions** à côté.

## Mod loaders

Un mod loader permet à Minecraft de charger des mods. Quand tu choisis une version, Misty demande à chaque loader s'il existe pour cette version, et n'affiche que ceux qui répondent oui. *Vanilla* (sans loader) est toujours disponible.

| Loader | Versions de Minecraft | Remarques |
|---|---|---|
| Forge | 1.7.10 et après | Le loader historique, une énorme bibliothèque de mods. |
| NeoForge | 1.20.1 et après | Fork de Forge pour les versions récentes. |
| Fabric | 1.14 et après | Léger et vite mis à jour. |
| Quilt | 1.14 et après | Fork de Fabric, charge aussi la plupart des mods Fabric. |
| OptiFine | 1.7.2 et après | Performances et shaders. N'installe pas d'autres mods. |
| LiteLoader | 1.5.2 à 1.12.2 | Pour les anciennes versions. |

Ces plages sont un premier filtre : un loader peut quand même manquer pour une version précise s'il n'est jamais sorti pour elle.

> [!NOTE]
> Les mods ne s'installent que sur un profil avec un mod loader (Forge, NeoForge, Fabric, Quilt ou LiteLoader). Les profils Vanilla et OptiFine acceptent les packs de ressources, les datapacks et, pour OptiFine, les shaders.

## Shaders

Minecraft ne sait pas charger de shaders tout seul. Misty t'indique ce qu'il faut quand tu installes un shader :

- **OptiFine** charge les shaders directement ;
- avec **Fabric** ou **NeoForge**, il faut le mod **Iris** ;
- avec **Forge**, il faut le mod **Oculus**.

Quand le mod nécessaire manque, Misty propose de l'installer avec le shader.

## Ce qui est téléchargé

Le premier lancement d'une version télécharge le jeu, ses bibliothèques et ressources, et la version de Java nécessaire. Le loader est installé par-dessus. Tout est partagé entre les profils : chaque version n'est téléchargée qu'une fois.
