+++
title = "Troubleshooting"
description = "The most common problems, what they mean and how to fix them. And how to report a bug."
section = "help"
+++

## The version list doesn't load

*Could not load versions* means Mojang's servers could not be reached. Check your Internet connection, then click **Reload the version list** next to the list.

## A mod loader is missing

Misty only offers the loaders that exist for the chosen version. NeoForge starts at 1.20.1, Fabric at 1.14, and LiteLoader stops at 1.12.2. See [Versions and mod loaders](/docs/versions-and-loaders).

## A mod won't install

| Message | What to do |
|---|---|
| *… has no version compatible with …* | The mod does not exist for this version and loader. Try another version of Minecraft or another loader. |
| *The author of … doesn't allow downloads from apps other than CurseForge* | Download the file from its CurseForge page and put it in the folder indicated. |
| *Pick a mod loader (Fabric, Forge…) to install mods* | The profile is Vanilla or OptiFine. Create a profile with a mod loader. |
| *Shaders not supported* | Use an OptiFine profile, or install Iris (Fabric, NeoForge) or Oculus (Forge). |

## The game doesn't start or crashes

1. Check the error shown in Misty (*Launch failed*, *Download failed*…).
2. Open the game folder and read `logs\latest.log`, or the newest file in `crash-reports\`. See [Files and folders](/docs/files-and-folders#game-logs).
3. If you use many mods, raise the memory in [Settings](/docs/settings).
4. Disable the most recently added mods one by one from the profile's **Content** tab.

## Microsoft sign-in fails

The message says what is wrong with the account: no Xbox profile, age verification, child account, game not owned… See [Accounts](/docs/accounts#when-sign-in-fails).

## "The game may still be running"

Some files cannot be deleted while Minecraft is open. Close the game, then try again.

## "Request limit reached"

Mojang temporarily limits the number of requests. Misty shows your last known skin and capes and retries by itself. See [Skin and capes](/docs/skins).

## Report a bug

Open an issue on [GitHub](https://github.com/Misty-MC/Misty/issues) with:

- the Misty version (Settings → About);
- the Minecraft version and the mod loader;
- what you did, what you expected and what happened;
- `logs\latest.log` and, after a crash, the report from `crash-reports\`.
