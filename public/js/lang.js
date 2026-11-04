// Gestion de la langue : détection, application des textes, menu déroulant.
import { TRANSLATIONS, LANGS } from "./i18n/index.js";
import { $, $$ } from "./dom.js";

const STORAGE_KEY = "misty-lang";
let current = "fr";
const listeners = [];

export const getLang = () => current;
export const t = () => TRANSLATIONS[current];
export const onLangChange = fn => listeners.push(fn);

function detect() {
  try { const s = localStorage.getItem(STORAGE_KEY); if (LANGS.includes(s)) return s; } catch (e) {}
  const n = (navigator.language || "en").slice(0, 2).toLowerCase();
  return LANGS.includes(n) ? n : "en";
}

export function setLang(l) {
  current = l;
  const tr = TRANSLATIONS[l];
  document.documentElement.lang = l;
  document.title = tr.title;
  $$("[data-i18n]").forEach(el => { el.innerHTML = tr[el.dataset.i18n] ?? ""; });
  $$("[data-i18n-alt]").forEach(el => { el.alt = tr[el.dataset.i18nAlt]; });

  // L'application n'existe qu'en FR/EN : ES/RU/IT utilisent les captures anglaises.
  const dir = l === "fr" ? "fr" : "en";
  $$("[data-shot]").forEach(el => {
    const src = `/assets/screens/${dir}/${el.dataset.shot}.png`;
    if (el.getAttribute("src") !== src) el.src = src;
  });

  $$("[data-lang]").forEach(b => b.setAttribute("aria-pressed", String(b.dataset.lang === l)));
  $("lang-cur").textContent = l.toUpperCase();
  try { localStorage.setItem(STORAGE_KEY, l); } catch (e) {}
  listeners.forEach(fn => fn(l));
}

function initDropdown() {
  const dd = $("langdd"), btn = dd.querySelector("button");
  const close = () => { dd.classList.remove("open"); btn.setAttribute("aria-expanded", "false"); };
  btn.addEventListener("click", e => {
    e.stopPropagation();
    btn.setAttribute("aria-expanded", String(dd.classList.toggle("open")));
  });
  document.addEventListener("click", close);
  document.addEventListener("keydown", e => { if (e.key === "Escape") close(); });
  $$("[data-lang]").forEach(b => b.addEventListener("click", () => { setLang(b.dataset.lang); close(); }));
}

export function initLang() {
  initDropdown();
  setLang(detect());
}
