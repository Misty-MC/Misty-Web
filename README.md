# Site Misty

Site du launcher Misty, entièrement servi par un serveur **Rust** ([axum](https://github.com/tokio-rs/axum)).
Les pages sont générées côté serveur avec [maud](https://maud.lambda.xyz/) : traductions, notes de version,
paysage en pixel art… Le seul JavaScript (`public/js/site.js`) ajoute le défilement fluide et quelques
animations. Sans lui, le site reste entièrement utilisable.

## Pages et routes

| Route | Rôle |
|---|---|
| `/` | Page d'accueil |
| `/patch-notes` | Toutes les versions et leurs notes (`?filter=stable` ou `?filter=pre`, ancre `#<tag>` pour ouvrir une version) |
| `/download/setup`, `/download/portable` | Dernier `Misty-Setup-<v>.exe` / `Misty-<v>.zip`, relayé depuis GitHub |
| `/api/latest` | Version, date, notes et tailles de la dernière version (JSON) |
| `?lang=fr\|en\|es\|ru\|it` | Change la langue, mémorisée dans un cookie. Sinon : cookie, puis langue du navigateur |

Les releases GitHub sont gardées en cache 2 minutes : une nouvelle release apparaît automatiquement.
Les fichiers doivent garder les noms `Misty-Setup-<v>.exe` et `Misty-<v>.zip`.

## Organisation

```
src/
├── main.rs            démarrage, routes, fichiers statiques
├── routes.rs          pages, /api/latest, /download/*
├── github.rs          releases GitHub + cache
├── i18n.rs            langues : choix et chargement des traductions
├── markdown.rs        notes de version Markdown → HTML sûr
├── format.rs          dates et tailles selon la langue
├── scene.rs           paysage pixel art (SVG généré au démarrage)
└── views/             pages HTML (maud)
    ├── layout.rs      <head>, navigation, pied de page
    ├── home.rs        accueil
    ├── patch_notes.rs patch notes
    ├── not_found.rs   404
    └── icons.rs       icônes SVG
locales/               une langue par fichier TOML (fr, en, es, ru, it)
public/
├── css/               base, layout, home, patch-notes
├── js/site.js         défilement fluide, nav, animations
└── assets/            icône, polices Silkscreen, captures FR/EN
Dockerfile             image de production
```

### Ajouter une langue
1. Copier `locales/en.toml` vers `locales/<code>.toml` et traduire les valeurs.
2. Ajouter la langue dans `src/i18n.rs` (enum `Lang`, `Lang::ALL`, `code()` et `TRANSLATIONS`).
3. `cargo test` vérifie qu'aucune clé ne manque.

## Lancer en local

```
cargo run
```

Le site est alors sur http://localhost:3000. Pour les tests : `cargo test`.

## Variables d'environnement

| Variable | Défaut | Rôle |
|---|---|---|
| `PORT` | `3000` | Port d'écoute |
| `GITHUB_TOKEN` | – | Token GitHub sans permission, pour éviter la limite de l'API (recommandé) |
| `MISTY_REPO` | `xTheoxreborn/Misty` | Dépôt dont on affiche les releases |
| `MISTY_PUBLIC_DIR` | `public` | Dossier des fichiers statiques |
| `GITHUB_API` | `https://api.github.com` | Pour tester avec une fausse API |

## Déploiement (Coolify ou tout hébergeur Docker)

1. Créer une ressource à partir de ce dépôt avec le **Build Pack : Dockerfile**.
2. Port exposé : `3000`.
3. (Optionnel) Ajouter la variable `GITHUB_TOKEN`.

Cloudflare Pages ne peut pas faire tourner ce serveur Rust : l'ancienne version (fonctions JavaScript) a été retirée.
