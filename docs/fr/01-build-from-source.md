+++
title = "Compiler Misty"
description = "Compiler le launcher, lancer ses tests, publier une release et s'y retrouver dans le code."
section = "dev"
+++

Misty est open source, sous licence MIT. Le code est sur [GitHub](https://github.com/Misty-MC/Misty) ; le projet Rust est dans le dossier `misty-rust`.

## Prérequis

- Windows 10 ou 11.
- [Rust](https://rustup.rs) (toolchain stable, cible MSVC) avec les outils de build C++ de Visual Studio.

## Clé API CurseForge

L'API CurseForge exige une clé personnelle, gratuite sur [console.curseforge.com](https://console.curseforge.com). Elle doit rester **privée** :

1. Colle-la dans `misty-rust/curseforge.key` (fichier ignoré par git).
2. Recompile : la clé est intégrée à l'exe au build.

Autre possibilité : la variable d'environnement `MISTY_CURSEFORGE_KEY`. Sans clé, l'onglet CurseForge explique comment la configurer ; Modrinth fonctionne normalement.

## Compiler et lancer

```powershell
cd misty-rust
cargo run              # version de développement
cargo build --release  # exe final : target\release\Misty.exe
```

## Tests

Les tests utilisent Internet et ton dossier Misty, mais ne lancent jamais le jeu :

```powershell
cargo test -- --ignored --nocapture
```

## Publier une release

1. Mets à jour `VERSION` dans `misty-rust/src/services/app_info.rs` et `version` dans `misty-rust/Cargo.toml`, puis ajoute l'entrée dans `CHANGELOG.md` (il est intégré à l'exe et affiché dans la page Notes de version).
2. Lance le script de release (il faut [Inno Setup 6](https://jrsoftware.org/isdl.php) : `winget install JRSoftware.InnoSetup`) :

   ```powershell
   powershell -ExecutionPolicy Bypass -File installer\build.ps1
   ```

   Il produit dans `publish\` : le dossier `Misty-<version>`, l'archive `Misty-<version>.zip` (utilisée par la mise à jour automatique) et l'installeur `Misty-Setup-<version>.exe`. `-SansInstallateur` ne génère que le dossier et le zip.
3. Crée une release GitHub avec le tag `<version>` et joins-y **le zip et l'installeur**.

> [!WARNING]
> `VERSION`, le tag GitHub (sans le `v`) et le nom du dossier `Misty-<version>` doivent être identiques. Sinon le launcher reproposera la mise à jour en boucle.

## S'y retrouver

| Dossier | Rôle |
|---|---|
| `misty-rust/ui/` | L'interface Slint : thème, composants et un fichier par page |
| `misty-rust/src/ui/` | Le code derrière chaque page |
| `misty-rust/src/minecraft/` | Installation et lancement du jeu : versions, Java, mod loaders |
| `misty-rust/src/services/` | La logique : réglages, comptes, profils, contenu, clients Modrinth et CurseForge, mises à jour… |
| `misty-rust/lang/` | Traductions (`en.json`, `fr.json`, `es.json`, `it.json`, `ru.json`), intégrées à l'exe |
| `installer/` | Script de release et installeur Inno Setup |

## Contribuer

Les signalements de bugs et les pull requests sont les bienvenus sur [GitHub](https://github.com/Misty-MC/Misty). Pour améliorer cette documentation, utilise le lien **Modifier cette page sur GitHub** en bas de chaque page : la doc se trouve dans le dépôt [Misty-Web](https://github.com/Misty-MC/Misty-Web), dans `docs/<langue>/`.
