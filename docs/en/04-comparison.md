+++
title = "Misty vs other launchers"
description = "Misty compared with Lunar Client, Badlion Client, Horizon, Modrinth App and the CurseForge app: features, accounts, mods and measured performance."
section = "start"
+++

There are many ways to launch Minecraft: Java Edition. They don't all do the same job, so here is an honest comparison to help you pick the right one. Misty doesn't try to replace a PvP client: it is a **light, general launcher** for every version and every mod loader.

> [!NOTE]
> This page compares what each launcher offers. Features change over time: if something here is out of date, tell us on [GitHub](https://github.com/Misty-MC/Misty/issues).

## At a glance

| | Misty | Lunar Client | Badlion Client | Horizon | Modrinth App | CurseForge app |
|---|---|---|---|---|---|---|
| **Kind** | General launcher | PvP client | PvP client | Rust reimplementation of the game + launcher | General launcher | General launcher |
| **Price** | Free | Free (paid cosmetics) | Free (paid cosmetics) | Free | Free (optional subscription) | Free |
| **Open source** | Yes (MIT) | No | No | Not checked | Yes (GPL-3.0) | No |
| **Technology** | Rust + Slint (native) | Electron | Electron | Rust | Tauri (Rust + web UI) | Electron |
| **Every Minecraft version** | Yes, down to alpha and indev | A chosen set of versions | A chosen set of versions | Its own versions | Yes | Yes |
| **Forge, NeoForge, Fabric, Quilt** | Yes, plus OptiFine and LiteLoader | No (built-in mods) | No (built-in mods) | Not checked | Yes | Yes |
| **Mods from Modrinth** | Yes | No | No | Yes | Yes | No |
| **Mods from CurseForge** | Yes | No | No | Not checked | No | Yes |
| **Modpacks** | Modrinth (.mrpack) and CurseForge | No | No | Import | Modrinth | CurseForge |
| **Microsoft account** | Yes, several | Yes | Yes | Yes | Yes | Yes |
| **Offline accounts** | Yes | No | No | Yes | No | No |
| **Texture pack editor** | Yes | No | No | No | No | No |
| **Cosmetics** | Coming soon (Misty cape and elytra to try) | Yes | Yes | No | No | No |
| **Windows / macOS / Linux** | Windows | All three | Windows, macOS | Not checked | All three | All three |

**In short:**

- You play **PvP on a few versions** and want built-in FPS mods: Lunar Client or Badlion Client.
- You want **every version, every loader and mods from both Modrinth and CurseForge**, in a small native app with offline accounts: **Misty**.
- You only use **Modrinth** content: the Modrinth App is excellent. Only **CurseForge** modpacks: the CurseForge app.
- You are curious about **another engine** for the game itself: Horizon reimplements Minecraft in Rust ([playhorizon.dev](https://playhorizon.dev)).

## Performance

Measured on a **Ryzen 7 7435HS, 24 GB of RAM, Windows 10**, Misty 0.3.4 (release build), on the home page, 15 seconds after launch. Average of 3 launches.

| | Misty |
|---|---|
| **Download size** | 38 MB (a single `Misty.exe`) |
| **Window shown after** | about 50 ms (the loading screen, then the home page) |
| **Memory at rest (working set)** | about 120 MB |
| **Memory at rest (private)** | about 170 MB |

> [!IMPORTANT]
> We only publish figures we measured ourselves. The other launchers haven't been measured on this machine yet, so they aren't in this table: we prefer an empty cell to a made-up number.

### Measure it yourself

The method is simple and works for any launcher:

1. Close every launcher and reboot, so nothing is cached in memory.
2. Start the launcher with a stopwatch, and note when its window appears.
3. Wait 15 seconds on its home page without touching anything.
4. In the **Task Manager** (Details tab), note the **Working set** and **Private bytes** of its processes (an Electron app runs several: add them up).
5. Do it three times and take the average.

Memory used by **the game itself** depends on Minecraft, your mods and the RAM you allow it ([Settings](/docs/settings)), not on the launcher, which is why it isn't compared here.
