// Infos de la dernière release pour la page d'accueil (version, tailles, notes).
const REPO = "xTheoxreborn/Misty";

export async function onRequestGet({ env }) {
  const headers = { "User-Agent": "misty-site", Accept: "application/vnd.github+json" };
  if (env.GITHUB_TOKEN) headers.Authorization = `Bearer ${env.GITHUB_TOKEN}`;
  const rel = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers,
    cf: { cacheTtl: 120, cacheEverything: true },
  });
  if (!rel.ok) return Response.json({ error: "unavailable" }, { status: 502 });
  const r = await rel.json();
  const size = (re) => r.assets.find((a) => re.test(a.name))?.size ?? null;
  return Response.json(
    {
      version: r.tag_name,
      date: r.published_at,
      notes: r.body || "",
      setupSize: size(/^Misty-Setup-.*\.exe$/i),
      portableSize: size(/^Misty-(?!Setup).*\.zip$/i),
    },
    { headers: { "Cache-Control": "public, max-age=120" } }
  );
}
