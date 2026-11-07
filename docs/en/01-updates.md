+++
title = "Updates"
description = "How Misty updates itself, and how to check for a new version by hand."
section = "help"
+++

Misty updates itself from its [GitHub releases](https://github.com/Misty-MC/Misty/releases), for both the installer and the portable version.

## Automatic updates

When Misty starts, it checks whether a newer version was released. If so, it shows **Update available** with the new and current version numbers:

- **Update** downloads the new version, installs it next to the current one and restarts Misty. The old version is then removed.
- **Later** keeps the current version; you will be asked again next time.

After the restart, Misty shows **Update complete**, removes the old version and points your shortcuts (and the *Installed apps* entry) to the new one. A custom `Icon.ico` is carried over.

Your data (profiles, worlds, accounts, settings) is not touched by an update.

## Check by hand

In [Settings](/docs/settings), the About card has a **Check for updates** button. It tells you whether you are already up to date.

## What changed?

- In the launcher: the **Release notes** page lists the changes of every version.
- On this site: the [patch notes](/patch-notes) page shows every release, with its files.

> [!NOTE]
> Misty never offers to "update" to an older version than the one installed.
