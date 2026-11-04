// Interactions de la page : nav au défilement, capture inclinée, bandeau, onglets, apparitions.
import { $, $$, reducedMotion } from "./dom.js";

export function initScrollEffects() {
  const header = document.querySelector("header");
  const stage = $("stage"), win = stage.querySelector(".window");
  function onScroll() {
    header.classList.toggle("scrolled", scrollY > 10);
    if (reducedMotion) return;
    const r = stage.getBoundingClientRect();
    const p = Math.min(1, Math.max(0, 1 - (r.top - innerHeight * .25) / (innerHeight * .6)));
    win.style.setProperty("--tilt", (14 * (1 - p)).toFixed(2) + "deg");
    win.style.setProperty("--sc", (.96 + .04 * p).toFixed(3));
  }
  addEventListener("scroll", onScroll, { passive: true });
  onScroll();
}

// Bandeau défilant : contenu dupliqué pour une boucle continue.
export function initMarquee() {
  const track = $("track");
  track.innerHTML += track.innerHTML;
}

// Vitrine à onglets : la barre de progression pilote le passage à l'onglet suivant
// (elle se met en pause au survol, et ne tourne pas si le mouvement est réduit).
export function initShowcase() {
  const show = $("showcase");
  const tabs = [...show.querySelectorAll(".tab")];
  const panes = show.querySelectorAll(".screens img");
  let cur = 0;

  function select(i) {
    cur = i;
    tabs.forEach((tab, k) => { tab.classList.toggle("on", k === i); tab.setAttribute("aria-selected", String(k === i)); });
    panes.forEach(p => p.classList.toggle("on", p.dataset.pane === tabs[i].dataset.tab));
    const bar = tabs[i].querySelector(".bar i");
    bar.style.animation = "none"; void bar.offsetWidth; bar.style.animation = "";
  }

  tabs.forEach((tab, i) => {
    tab.addEventListener("click", () => select(i));
    tab.querySelector(".bar i").addEventListener("animationend", () => { if (cur === i) select((i + 1) % tabs.length); });
  });
  show.addEventListener("mouseenter", () => show.classList.add("paused"));
  show.addEventListener("mouseleave", () => show.classList.remove("paused"));
  select(0);
}

export function initReveal() {
  const items = $$(".reveal");
  if (!("IntersectionObserver" in window)) { items.forEach(el => el.classList.add("in")); return; }
  const io = new IntersectionObserver(entries => entries.forEach(e => {
    if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
  }), { threshold: .12 });
  items.forEach(el => io.observe(el));
}

export function initFooterYear() {
  $("year").textContent = new Date().getFullYear();
}
