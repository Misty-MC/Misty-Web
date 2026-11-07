+++
title = "Build from source"
description = "Compile the launcher, run its tests, publish a release, and find your way around the code."
section = "dev"
+++

Misty is open source under the MIT license. The code is on [GitHub](https://github.com/Misty-MC/Misty); the Rust project is in the `misty-rust` folder.

## Requirements

- Windows 10 or 11.
- [Rust](https://rustup.rs) (stable toolchain, MSVC target) with the Visual Studio C++ build tools.

## CurseForge API key

The CurseForge API needs a personal key, free on [console.curseforge.com](https://console.curseforge.com). It must stay **private**:

1. Paste it into `misty-rust/curseforge.key` (the file is ignored by git).
2. Rebuild: the key is embedded in the exe at build time.

Alternatively, set the `MISTY_CURSEFORGE_KEY` environment variable. Without a key, the CurseForge tab explains how to set one up; Modrinth works normally.

## Build and run

```powershell
cd misty-rust
cargo run              # development build
cargo build --release  # final exe: target\release\Misty.exe
```

## Tests

The tests use the Internet and your Misty folder, but never start the game:

```powershell
cargo test -- --ignored --nocapture
```

## Publish a release

1. Update `VERSION` in `misty-rust/src/services/app_info.rs` and `version` in `misty-rust/Cargo.toml`, then add an entry to `CHANGELOG.md` (it is embedded in the exe and shown on the Release notes page).
2. Run the release script (it needs [Inno Setup 6](https://jrsoftware.org/isdl.php): `winget install JRSoftware.InnoSetup`):

   ```powershell
   powershell -ExecutionPolicy Bypass -File installer\build.ps1
   ```

   It produces in `publish\`: the `Misty-<version>` folder, the `Misty-<version>.zip` archive (used by the auto-update) and the `Misty-Setup-<version>.exe` installer. `-SansInstallateur` only builds the folder and the zip.
3. Create a GitHub release with the tag `<version>` and attach **the zip and the installer**.

> [!WARNING]
> `VERSION`, the GitHub tag (without a `v`) and the `Misty-<version>` folder name must be identical. Otherwise the launcher will offer the update over and over.

## Find your way around

| Folder | Role |
|---|---|
| `misty-rust/ui/` | The Slint interface: theme, widgets and one file per page |
| `misty-rust/src/ui/` | The code behind each page |
| `misty-rust/src/minecraft/` | Game installation and launch: versions, Java, mod loaders |
| `misty-rust/src/services/` | The logic: settings, accounts, profiles, content, Modrinth and CurseForge clients, updates… |
| `misty-rust/lang/` | Translations (`en.json`, `fr.json`, `es.json`, `it.json`, `ru.json`), embedded in the exe |
| `installer/` | Release script and Inno Setup installer |

## Contribute

Bug reports and pull requests are welcome on [GitHub](https://github.com/Misty-MC/Misty). To improve this documentation, use the **Edit this page on GitHub** link at the bottom of each page: the docs live in the [Misty-Web](https://github.com/Misty-MC/Misty-Web) repository, in `docs/<language>/`.
