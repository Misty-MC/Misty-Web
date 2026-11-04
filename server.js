// Serveur Node (sans dépendance) pour héberger le site hors Cloudflare (ex. Coolify).
// Reprend le comportement de functions/ : /api/latest, /download/setup, /download/portable + fichiers statiques de public/.
const http = require("http");
const fs = require("fs");
const path = require("path");
const { Readable } = require("stream");

const REPO = "xTheoxreborn/Misty";
const PORT = process.env.PORT || 3000;
const PUBLIC = path.join(__dirname, "public");
const MATCH = {
  setup: /^Misty-Setup-.*\.exe$/i,
  portable: /^Misty-(?!Setup).*\.zip$/i,
};
const MIME = {
  ".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript",
  ".png": "image/png", ".ico": "image/x-icon", ".ttf": "font/ttf", ".txt": "text/plain; charset=utf-8",
  ".svg": "image/svg+xml", ".json": "application/json",
};

const ghHeaders = { "User-Agent": "misty-site", Accept: "application/vnd.github+json" };
if (process.env.GITHUB_TOKEN) ghHeaders.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

let cache = { at: 0, data: null };
async function latestRelease() {
  if (cache.data && Date.now() - cache.at < 120000) return cache.data;
  const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers: ghHeaders });
  if (!res.ok) return null;
  cache = { at: Date.now(), data: await res.json() };
  return cache.data;
}

function send(res, status, body, type = "text/plain; charset=utf-8", extra = {}) {
  res.writeHead(status, { "Content-Type": type, ...extra });
  res.end(body);
}

async function handle(req, res) {
  const url = new URL(req.url, "http://x");
  const p = decodeURIComponent(url.pathname);

  if (p === "/api/latest") {
    const r = await latestRelease();
    if (!r) return send(res, 502, JSON.stringify({ error: "unavailable" }), "application/json");
    const size = (re) => r.assets.find((a) => re.test(a.name))?.size ?? null;
    return send(res, 200, JSON.stringify({
      version: r.tag_name,
      date: r.published_at,
      notes: r.body || "",
      setupSize: size(MATCH.setup),
      portableSize: size(MATCH.portable),
    }), "application/json", { "Cache-Control": "public, max-age=120" });
  }

  const dl = p.match(/^\/download\/(setup|portable)$/);
  if (dl) {
    const r = await latestRelease();
    if (!r) return send(res, 502, "Release introuvable");
    const asset = r.assets.find((a) => MATCH[dl[1]].test(a.name));
    if (!asset) return send(res, 404, "Fichier introuvable dans la dernière release");
    const file = await fetch(asset.browser_download_url, { headers: { "User-Agent": "misty-site" } });
    if (!file.ok) return send(res, 502, "Téléchargement impossible");
    res.writeHead(200, {
      "Content-Type": asset.content_type || "application/octet-stream",
      "Content-Length": String(asset.size),
      "Content-Disposition": `attachment; filename="${asset.name}"`,
      "Cache-Control": "public, max-age=300",
    });
    return Readable.fromWeb(file.body).pipe(res);
  }

  // Fichiers statiques
  let file = path.normalize(path.join(PUBLIC, p === "/" ? "index.html" : p));
  if (!file.startsWith(PUBLIC)) return send(res, 403, "Interdit");
  fs.stat(file, (err, st) => {
    if (err || !st.isFile()) return send(res, 404, "Introuvable");
    res.writeHead(200, {
      "Content-Type": MIME[path.extname(file).toLowerCase()] || "application/octet-stream",
      "Cache-Control": "public, max-age=3600",
    });
    fs.createReadStream(file).pipe(res);
  });
}

http.createServer((req, res) => {
  handle(req, res).catch(() => { if (!res.headersSent) send(res, 500, "Erreur serveur"); else res.end(); });
}).listen(PORT, () => console.log(`Misty-site sur le port ${PORT}`));
