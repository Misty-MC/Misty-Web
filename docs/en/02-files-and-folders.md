+++
title = "Files and folders"
description = "Where Misty stores the game, your profiles, your accounts and its settings."
section = "help"
+++

## The data folder

Everything Misty downloads or creates goes to **`%LOCALAPPDATA%\Misty`**. Paste this path into the Windows Explorer address bar to open it, or use **Open the game folder** in the sidebar.

| File / folder | Content |
|---|---|
| `data.txt` | Settings: last account and version, RAM, language, shown versions, Rich Presence, last profile… |
| `listpseudo.txt` | Offline accounts |
| `msa.json` | Signed-in Microsoft accounts |
| `instances\<id>\` | One folder per [profile](/docs/profiles) |
| `versions\`, `libraries\`, `assets\`, Java | The Minecraft installation, shared by every profile |

## A profile's folder

Each profile in `instances\<id>\` contains:

| File / folder | Content |
|---|---|
| `instance.json` | The profile's settings: name, version, mod loader… |
| `mods\` | Mods |
| `resourcepacks\` | Resource packs |
| `shaderpacks\` | Shaders |
| `saves\` | Worlds |
| `misty-contenu.json` | What Misty knows about the installed content |

**Open the profile folder**, on the profile's page, opens it directly.

## Game logs

Minecraft writes its logs in the game folder it runs in (the profile's folder, or the data folder with *Quick play*):

- `logs\latest.log`: the log of the last game;
- `crash-reports\`: one report per crash.

Attach these files when you [report a problem](/docs/troubleshooting#report-a-bug).

## The application folder

- Installer: `%LOCALAPPDATA%\Programs\Misty\Misty-<version>\Misty.exe`
- Portable: wherever you extracted the zip.

> [!TIP]
> To give Misty your own icon, put an `Icon.ico` file next to `Misty.exe`.
