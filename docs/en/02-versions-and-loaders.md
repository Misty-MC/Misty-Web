+++
title = "Versions and mod loaders"
description = "Which Minecraft versions Misty shows, and which mod loaders it offers for each of them."
section = "play"
+++

## Minecraft versions

Misty lists the versions published by Mojang. **Releases** are always shown. The other types can be turned on in the **Shown versions** card of [Settings](/docs/settings):

- **Snapshots**: test versions of the next release;
- **Betas**;
- **Alphas and Indev**: the very first versions of the game.

If the list could not be loaded (no connection, for example), use the **Reload the version list** button next to it.

## Mod loaders

A mod loader is what lets Minecraft load mods. When you pick a version, Misty asks each loader whether it exists for that version, and only shows the ones that do. *Vanilla* (no loader) is always available.

| Loader | Minecraft versions | Notes |
|---|---|---|
| Forge | 1.7.10 and later | The historical loader, a huge mod library. |
| NeoForge | 1.20.1 and later | Fork of Forge for recent versions. |
| Fabric | 1.14 and later | Light and quick to update. |
| Quilt | 1.14 and later | Fork of Fabric, also loads most Fabric mods. |
| OptiFine | 1.7.2 and later | Performance and shaders. Does not install other mods. |
| LiteLoader | 1.5.2 to 1.12.2 | For older versions. |

These ranges are a first filter: a loader may still be missing for a given version if it was never released for it.

> [!NOTE]
> Mods can only be installed on a profile with a mod loader (Forge, NeoForge, Fabric, Quilt or LiteLoader). Vanilla and OptiFine profiles accept resource packs, datapacks and, for OptiFine, shaders.

## Shaders

Minecraft cannot load shaders by itself. Misty tells you what is needed when you install a shader:

- **OptiFine** loads shaders directly;
- with **Fabric** or **NeoForge**, the **Iris** mod is needed;
- with **Forge**, the **Oculus** mod is needed.

When the required mod is missing, Misty offers to install it with the shader.

## What gets downloaded

The first launch of a version downloads the game, its libraries and assets, and the Java version it needs. The loader is installed on top. Everything is shared between profiles, so each version is downloaded only once.
