+++
title = "Modpacks"
description = "Install a Modrinth or CurseForge modpack as a ready-to-play profile."
section = "play"
+++

A modpack is a set of mods and settings put together by its author. Misty turns it into a new [profile](/docs/profiles), with the right Minecraft version and mod loader.

## Install a modpack

1. Open **Profiles, mods and modpacks**, then click **Install a modpack**.
2. Pick the platform, **Modrinth** or **CurseForge**, and search.
3. Choose the **version** of the modpack you want.
4. Confirm: a new profile is created and its files are downloaded.

Misty installs the mod loader version the modpack asks for, copies its configuration files and downloads every mod.

Modrinth modpacks use the `.mrpack` format.

## Manual downloads

Some CurseForge authors only allow downloads from the CurseForge website. When a modpack contains such files, Misty shows the list after the installation:

1. Click **Open the pages**: each file's page opens in your browser.
2. Download the files.
3. Put each one in the folder indicated next to its name (usually the profile's `mods` folder).

If the whole modpack is restricted this way, Misty tells you it cannot be downloaded outside CurseForge.

## Common errors

| Message | Meaning |
|---|---|
| *This modpack has no published version* | The author has not published a file yet. |
| *The modpack doesn't specify its Minecraft version* | The modpack is incomplete; try another version of it. |
| *Unsupported mod loader* | The modpack uses a loader Misty does not install. |
