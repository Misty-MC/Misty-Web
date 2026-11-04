# Site Misty

## Organisation

```
public/                     le site statique (servi tel quel)
├── index.html              structure de la page (aucun style ni script en ligne)
├── css/
│   ├── base.css            polices, variables de couleur, reset, boutons, utilitaires
│   ├── layout.css          en-tête / navigation et pied de page
│   ├── sections.css        sections de l'accueil (hero, vitrine, étapes, téléchargement, FAQ…)
│   └── responsive.css      animations, mouvement réduit et points de rupture (chargé en dernier)
├── js/
│   ├── main.js             point d'entrée (module ES)
│   ├── lang.js             détection et changement de langue, menu déroulant
│   ├── release.js          dernière release : version, tailles, notes (via /api/latest)
│   ├── markdown.js         mini-rendu sécurisé des notes de release
│   ├── ui.js               nav au défilement, capture inclinée, bandeau, onglets, apparitions
│   ├── dom.js              raccourcis DOM partagés
│   └── i18n/               une langue par fichier (fr, en, es, ru, it) + index.js
└── assets/                 icône, polices Silkscreen, captures d'écran FR/EN

functions/                  Cloudflare Pages Functions
├── api/latest.js           version, taille et notes de la dernière release
└── download/[type].js      /download/setup et /download/portable — renvoient le dernier asset depuis ton domaine

server.js                   serveur Node sans dépendance (alternative à Cloudflare, ex. Coolify)
```

### Ajouter une langue
1. Copier `public/js/i18n/en.js` vers `public/js/i18n/<code>.js` et traduire les valeurs.
2. L'importer dans `public/js/i18n/index.js`.
3. Ajouter un bouton `data-lang="<code>"` dans le menu de la nav et dans le footer (`index.html`).

## Lancer en local
```
npm start        # http://localhost:3000
```

## Déploiement (Cloudflare Pages, gratuit)
1. Pousse ce dossier sur un dépôt GitHub (ex. `Misty-site`).
2. Cloudflare Dashboard → Workers & Pages → Create → Pages → Connect to Git.
3. Build command : vide. Output directory : `public`.
4. (Optionnel) Variable `GITHUB_TOKEN` (token sans permission) pour éviter la limite de l'API GitHub.

Chaque nouvelle release GitHub est prise en compte automatiquement (cache ~2 min).
Les assets doivent garder les noms `Misty-Setup-<v>.exe` et `Misty-<v>.zip`.
