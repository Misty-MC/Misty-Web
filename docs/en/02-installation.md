+++
title = "Installation"
description = "Installer or portable version: how to install Misty, where it goes and how to uninstall it."
section = "start"
+++

Misty comes in two forms, both available on the [download page](/#download) and in the [GitHub releases](https://github.com/Misty-MC/Misty/releases). Both update themselves.

## Installer (recommended)

1. Download `Misty-Setup-<version>.exe`.
2. Run it and follow the steps. You can choose to add a desktop shortcut.
3. Start Misty from the Start menu.

What the installer does:

- installs Misty in `%LOCALAPPDATA%\Programs\Misty`, **without administrator rights**;
- adds a Start menu shortcut (and a desktop one if you asked for it);
- registers Misty in Windows' **Installed apps**, so you can uninstall it cleanly.

## Portable version

1. Download `Misty-<version>.zip`.
2. Extract it wherever you like, for example on a USB drive.
3. Run `Misty-<version>\Misty.exe`.

Nothing is installed in Windows. The game data still goes to `%LOCALAPPDATA%\Misty` on the PC you play on. See [Files and folders](/docs/files-and-folders).

> [!TIP]
> Both versions use the same data folder: you can switch from one to the other without losing your profiles, worlds or settings.

## Nothing else to install

`Misty.exe` contains everything it needs: fonts, translations and images. Java is downloaded automatically when a Minecraft version needs it.

## Uninstall

- **Installer:** uninstall Misty from **Settings → Apps → Installed apps** in Windows. Your game data (versions, profiles, worlds) is kept unless you ask for it to be deleted.
- **Portable:** delete the extracted folder.

To erase **all** game data as well, use **Delete everything** in the Storage card of [Settings](/docs/settings) before uninstalling, or delete `%LOCALAPPDATA%\Misty` yourself.

> [!WARNING]
> Deleting the data folder also deletes the worlds of every profile. Back up the `saves` folders you want to keep first.
