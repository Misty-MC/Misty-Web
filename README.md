# Site Misty

- `public/` : le site (HTML + polices + icône)
- `functions/download/[type].js` : `/download/setup` et `/download/portable` — récupèrent le dernier asset de la release GitHub et le renvoient depuis ton domaine
- `functions/api/latest.js` : version, taille et notes de la dernière release

## Déploiement (Cloudflare Pages, gratuit)
1. Pousse ce dossier sur un dépôt GitHub (ex. `Misty-site`).
2. Cloudflare Dashboard → Workers & Pages → Create → Pages → Connect to Git.
3. Build command : vide. Output directory : `public`.
4. (Optionnel) Variable `GITHUB_TOKEN` (token sans permission) pour éviter la limite de l'API GitHub.

Chaque nouvelle release GitHub est prise en compte automatiquement (cache ~2 min).
Les assets doivent garder les noms `Misty-Setup-<v>.exe` et `Misty-<v>.zip`.
