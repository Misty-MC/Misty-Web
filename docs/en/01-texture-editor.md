+++
title = "Texture pack editor"
description = "Create a resource pack by editing Minecraft's textures pixel by pixel, then export it as a .zip."
section = "tools"
+++

> [!WARNING]
> The texture pack editor is **experimental**: you may run into bugs.

Open **Texture packs** in the sidebar to see your packs, edit them, export them or delete them.

## Create a pack

1. Click **Create a pack**.
2. Give it a **name** and a **description**.
3. Pick the **Minecraft version** it is for. Resource packs exist since version 1.6.1.
4. Click **Create pack**.

The original textures of that version are downloaded **once**, then reused for every pack of the same version.

## Edit textures

The editor shows every texture of the game, sorted into categories (building blocks, redstone, entities, interface…) with a counter. Use the search box to find one by name.

- **All textures**: every texture of the version.
- **My textures**: only the ones you edited.

Click a texture to open it, then draw.

### Tools and shortcuts

| Tool | Shortcut |
|---|---|
| Pencil | `B` |
| Eraser | `E` |
| Paint bucket | `G` |
| Color picker | `I`, or right-click for a quick pick |
| Undo / Redo | `Ctrl+Z` / `Ctrl+Y` |
| Save | `Ctrl+S` |
| Zoom | `Ctrl` + mouse wheel |

The side panel shows the **recent colors** and the **colors of the texture**. The **Grid** button shows the pixel grid. **Reset** restores the original texture and removes your version from the pack.

The texture you are working on is saved when you leave the page.

### External editor

Prefer Paint.NET or another program? Click **External editor** and choose your program (for example `paintdotnet.exe`; cancel to use the default app). Edit and save in your program: Misty reloads the texture automatically when the file changes.

## Pack icon

In the editor, **Change the pack icon** sets the image shown in Minecraft's resource pack list.

## Export and use the pack

1. Click **Export** and choose where to save the `.zip`.
2. Put the zip in the `resourcepacks` folder of a [profile](/docs/profiles).
3. In Minecraft, enable it in **Options → Resource Packs**.

Misty writes the right `pack_format` for the version, including very old and very recent ones.
