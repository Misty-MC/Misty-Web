// Dernière release (via /api/latest) : version, tailles des fichiers et notes.
import { $, $$ } from "./dom.js";
import { t } from "./lang.js";
import { renderMarkdown } from "./markdown.js";

let rel = null, failed = false;

function fmtSize(bytes) {
  if (!bytes) return "";
  const tr = t();
  return (bytes / 1048576).toFixed(1).replace(".", tr.locale === "en-US" ? "." : ",") + " " + tr.mb;
}

export function renderRelease() {
  const tr = t();
  if (rel) {
    const date = new Date(rel.date).toLocaleDateString(tr.locale, { day: "numeric", month: "long", year: "numeric" });
    const setup = fmtSize(rel.setupSize), portable = fmtSize(rel.portableSize);
    $$(".ver-txt").forEach(el => { el.textContent = rel.version; });
    $("size-setup").textContent = `${rel.version} · ${setup} · Windows 10/11`;
    $("size-portable").textContent = `${rel.version} · ${portable} · .zip`;
    $("meta-setup").textContent = `${tr.version} ${rel.version} · ${setup} · .exe`;
    $("meta-portable").textContent = `${tr.version} ${rel.version} · ${portable} · .zip`;
    $("log-title").textContent = `${tr.version} ${rel.version} · ${date}`;
    const notes = (rel.notes || "").trim();
    $("log-body").innerHTML = notes ? renderMarkdown(notes) : `<p>${tr.notes_default}</p>`;
  } else if (failed) {
    $$(".ver-txt").forEach(el => { el.textContent = tr.latest_fallback; });
    $("log-title").textContent = tr.unavailable;
    $("log-body").innerHTML = `<p>${tr.notes_default}</p>`;
  } else {
    $("log-title").textContent = tr.loading;
  }
}

export function loadRelease() {
  fetch("/api/latest")
    .then(r => r.json())
    .then(d => { if (d.error) throw new Error(d.error); rel = d; })
    .catch(() => { failed = true; })
    .finally(renderRelease);
}
