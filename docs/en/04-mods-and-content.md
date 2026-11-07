+++
title = "Mods, packs and shaders"
description = "Find content on Modrinth or CurseForge, install it with its dependencies and manage what a profile contains."
section = "play"
+++

Content is installed per [profile](/docs/profiles). Open a profile, then use its two tabs: **Discover** to find content, **Content** to manage what is installed.

## Discover

1. Choose the platform: **Modrinth** or **CurseForge**.
2. Choose the type: **Mods**, **Resource packs**, **Datapacks** or **Shaders**.
3. Search, and sort by relevance, downloads, popularity, newest, recently updated, featured or name.
4. Click **Install**.

Results are filtered to stay **compatible** with the profile's version and mod loader. When a mod needs other mods, Misty installs its **dependencies** too and tells you how many.

> [!NOTE]
> Mods need a mod loader. On a Vanilla or OptiFine profile, Misty asks you to pick a loader first. See [Versions and mod loaders](/docs/versions-and-loaders).

### Datapacks

Datapacks are installed **into a world**. Misty asks which world to use. If the profile has no world yet, launch it once, create a world, then come back.

### Shaders

Shaders need OptiFine, or the Iris (Fabric, NeoForge) or Oculus (Forge) mod. If it is missing, Misty offers to install it with the shader. See [Shaders](/docs/versions-and-loaders#shaders).

## Content

The **Content** tab lists everything installed in the profile, with a filter by type and a search box. For each item you can:

- **enable or disable** it: the file is renamed with a `.disabled` extension and the game ignores it;
- **delete** it.

Files you added to the folders by hand are identified too, by their hash on Modrinth or their fingerprint on CurseForge, so they show with their real name.

## CurseForge

Some CurseForge authors do not allow downloads from other apps than CurseForge. In that case Misty lists the files and opens their pages: download them and put them in the folder indicated. See [Modpacks](/docs/modpacks#manual-downloads).
