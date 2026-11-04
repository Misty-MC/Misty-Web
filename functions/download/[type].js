// /download/setup  -> dernier Misty-Setup-<version>.exe
// /download/portable -> dernier Misty-<version>.zip
// Le fichier est récupéré côté serveur et renvoyé tel quel : l'utilisateur ne quitte jamais le site.
const REPO = "xTheoxreborn/Misty";
const MATCH = {
  setup: /^Misty-Setup-.*\.exe$/i,
  portable: /^Misty-(?!Setup).*\.zip$/i,
};

export async function onRequestGet({ params, env, request }) {
  const matcher = MATCH[params.type];
  if (!matcher) return new Response("Type inconnu", { status: 404 });

  const headers = { "User-Agent": "misty-site", Accept: "application/vnd.github+json" };
  if (env.GITHUB_TOKEN) headers.Authorization = `Bearer ${env.GITHUB_TOKEN}`;

  const rel = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers,
    cf: { cacheTtl: 120, cacheEverything: true },
  });
  if (!rel.ok) return new Response("Release introuvable", { status: 502 });
  const asset = (await rel.json()).assets.find((a) => matcher.test(a.name));
  if (!asset) return new Response("Fichier introuvable dans la dernière release", { status: 404 });

  const file = await fetch(asset.browser_download_url, {
    headers: { "User-Agent": "misty-site" },
    cf: { cacheTtl: 3600, cacheEverything: true },
  });
  if (!file.ok) return new Response("Téléchargement impossible", { status: 502 });

  return new Response(file.body, {
    headers: {
      "Content-Type": asset.content_type || "application/octet-stream",
      "Content-Length": String(asset.size),
      "Content-Disposition": `attachment; filename="${asset.name}"`,
      "Cache-Control": "public, max-age=300",
    },
  });
}
