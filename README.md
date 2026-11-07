<div align="center">

<img src="public/assets/icone.png" alt="Misty" width="110" />

# Misty Web

**The website of Misty, the Minecraft launcher light as the mist.**

Rendered server-side in Rust: pages, translations, patch notes and the pixel-art night scene.

[![Website](https://img.shields.io/badge/website-misty--launcher.online-8ab4f8?style=flat-square)](https://misty-launcher.online/)
[![Deployment](https://img.shields.io/github/deployments/Misty-MC/Misty-Web/production?label=deploy&style=flat-square)](https://github.com/Misty-MC/Misty-Web/deployments)
[![Rust](https://img.shields.io/badge/written%20in-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![axum](https://img.shields.io/badge/server-axum-6f9cff?style=flat-square)](https://github.com/tokio-rs/axum)
[![Docker](https://img.shields.io/badge/deploy-Docker-2496ED?style=flat-square&logo=docker&logoColor=white)](Dockerfile)
[![License](https://img.shields.io/badge/license-MIT-success?style=flat-square)](LICENSE)

[**Website**](https://misty-launcher.online/) · [**Patch notes**](https://misty-launcher.online/patch-notes) · [**Launcher**](https://github.com/Misty-MC/Misty) · [**Releases**](https://github.com/Misty-MC/Misty/releases)

</div>

---

## ✨ Features

- **Home page**: hero with the latest version, screenshots, features, mod loaders, downloads and FAQ.
- **Patch notes** (`/patch-notes`): every release from GitHub, with stable / pre-release filters, notes you can expand and each version's files.
- **Direct downloads**: `/download/setup` and `/download/portable` relay the latest installer or zip from GitHub, so visitors never leave the site.
- **5 languages**: Français, English, Español, Русский, Italiano, chosen server-side (`?lang=`, then a cookie, then the browser's language).
- **Animated night scene**: misty mountains, a floating moon and drifting fog, generated in Rust as pixel-art SVG.
- **Light**: no front-end framework. A single small script adds smooth scrolling and animations, and the site still works without it.

## 🚀 Getting started

```bash
cargo run          # http://localhost:3000
cargo test         # translations, Markdown safety, dates, scene…
```

Requires [Rust](https://rustup.rs) (stable, edition 2024).

## ⚙️ Configuration

| Variable | Default | Purpose |
|---|---|---|
| `PORT` | `3000` | Port to listen on |
| `GITHUB_TOKEN` | – | Token with no permissions, to avoid GitHub API rate limits (recommended) |
| `MISTY_REPO` | `Misty-MC/Misty` | Repository whose releases are shown |
| `MISTY_PUBLIC_DIR` | `public` | Static files folder |
| `GITHUB_API` | `https://api.github.com` | Point to a fake API for testing |

Releases are cached for 2 minutes, so a new release shows up on its own.
Release files must keep the names `Misty-Setup-<version>.exe` and `Misty-<version>.zip`.

## 📦 Deployment

The site ships as a Docker image ([`Dockerfile`](Dockerfile)) and runs on [Coolify](https://coolify.io):
**Build Pack: Dockerfile**, exposed port **3000**.

Each push to `main` can be deployed through [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml):
the workflow starts the Coolify deployment, waits for it to finish and reports the result in the
**Deployments** panel of this repository. It needs:

| Where (Settings → Secrets and variables → Actions) | Name | Value |
|---|---|---|
| Variable | `COOLIFY_URL` | URL of the Coolify instance, e.g. `https://coolify.example.com` |
| Variable | `COOLIFY_APP_UUID` | UUID of the application in Coolify |
| Secret | `COOLIFY_TOKEN` | Coolify API token with the *deploy* permission |

Until these are set, the workflow is skipped.

<details>
<summary><b>🗂️ Project structure</b></summary>

```
src/
├── main.rs            startup, routes, static files
├── routes.rs          pages, /api/latest, /download/*
├── github.rs          GitHub releases + cache
├── i18n.rs            language negotiation and translations
├── markdown.rs        release notes Markdown → safe HTML
├── format.rs          localized dates and sizes
├── scene.rs           pixel-art night scene and fog (SVG)
└── views/             HTML pages (maud)
    ├── layout.rs      <head>, navigation, footer
    ├── home.rs        home page
    ├── patch_notes.rs patch notes
    ├── not_found.rs   404
    └── icons.rs       SVG icons
locales/               one TOML file per language
public/
├── css/               base, layout, home, patch-notes
├── js/site.js         smooth scrolling, navigation, animations
└── assets/            icon, Silkscreen font, screenshots
```

</details>

<details>
<summary><b>🌍 Adding a language</b></summary>

1. Copy `locales/en.toml` to `locales/<code>.toml` and translate the values.
2. Register the language in `src/i18n.rs` (`Lang` enum, `Lang::ALL`, `code()` and `TRANSLATIONS`).
3. Run `cargo test`: it fails if a key is missing.

</details>

## 📄 License

The code is released under the [MIT License](LICENSE).
The Silkscreen font is distributed under the [SIL Open Font License](public/assets/OFL.txt).

---

<div align="center">
<sub>
Misty is an independent launcher and is not affiliated with Mojang Studios or Microsoft.<br/>
Minecraft is a trademark of Mojang AB.
</sub>
</div>
