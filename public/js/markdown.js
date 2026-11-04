// Mini-rendu Markdown pour les notes de release GitHub (titres, listes, gras, code).
// Le texte est échappé avant toute mise en forme : aucun HTML de la release n'est injecté.
const escapeHtml = s => s.replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
const inline = s => escapeHtml(s).replace(/\*\*(.+?)\*\*/g, "<b>$1</b>").replace(/`([^`]+)`/g, "<code>$1</code>");

export function renderMarkdown(src) {
  let out = "", inList = false;
  for (const raw of src.split(/\r?\n/)) {
    const line = raw.trim();
    const li = line.match(/^[-*+]\s+(.*)/);
    if (li) {
      if (!inList) { out += "<ul>"; inList = true; }
      out += `<li>${inline(li[1])}</li>`;
      continue;
    }
    if (inList) { out += "</ul>"; inList = false; }
    if (!line) continue;
    const h = line.match(/^#{1,6}\s+(.*)/);
    out += h ? `<h4>${inline(h[1])}</h4>` : `<p>${inline(line)}</p>`;
  }
  return out + (inList ? "</ul>" : "");
}
