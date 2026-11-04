// Défilement fluide à la molette (inertie douce) et sur les liens d'ancre.
// Désactivé si l'utilisateur demande moins d'animations ou sur écran tactile (le défilement natif y est déjà fluide).
import { reducedMotion } from "./dom.js";

const EASE = 0.1;          // part du chemin restant parcourue à chaque image (plus petit = plus doux)
const HEADER_OFFSET = 96;  // même valeur que scroll-padding-top dans base.css

let target = 0, current = 0, raf = 0;

const maxScroll = () => document.documentElement.scrollHeight - innerHeight;
const clamp = v => Math.max(0, Math.min(maxScroll(), v));
const jump = y => scrollTo({ top: y, behavior: "instant" });

function frame() {
  current += (target - current) * EASE;
  if (Math.abs(target - current) < 0.5) {
    current = target;
    jump(current);
    raf = 0;
    return;
  }
  jump(current);
  raf = requestAnimationFrame(frame);
}

function animateTo(y) {
  if (!raf) current = scrollY;
  target = clamp(y);
  if (!raf) raf = requestAnimationFrame(frame);
}

// Un élément interne (ex. notes de version) peut-il encore défiler dans ce sens ? Alors on le laisse faire.
function innerScrollable(el, dy) {
  for (; el && el !== document.body && el !== document.documentElement; el = el.parentElement) {
    const oy = getComputedStyle(el).overflowY;
    if ((oy === "auto" || oy === "scroll") && el.scrollHeight > el.clientHeight) {
      if (dy > 0 ? el.scrollTop + el.clientHeight < el.scrollHeight - 1 : el.scrollTop > 0) return true;
    }
  }
  return false;
}

function onWheel(e) {
  if (e.ctrlKey || e.defaultPrevented) return; // zoom navigateur
  let dy = e.deltaY;
  if (e.deltaMode === 1) dy *= 16;
  else if (e.deltaMode === 2) dy *= innerHeight;
  if (!dy || Math.abs(e.deltaX) > Math.abs(dy) || innerScrollable(e.target, dy)) return;
  e.preventDefault();
  animateTo((raf ? target : scrollY) + dy);
}

function onAnchorClick(e) {
  const a = e.target.closest('a[href^="#"]');
  if (!a || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey) return;
  const id = a.getAttribute("href").slice(1);
  const el = id ? document.getElementById(id) : null;
  const y = !el || id === "top" ? 0 : el.getBoundingClientRect().top + scrollY - HEADER_OFFSET;
  e.preventDefault();
  history.pushState(null, "", "#" + id);
  animateTo(y);
}

export function initSmoothScroll() {
  if (reducedMotion) return;
  addEventListener("click", onAnchorClick);
  if (!matchMedia("(pointer: fine)").matches) return;
  addEventListener("wheel", onWheel, { passive: false });
  // Barre de défilement, clavier… : on se recale sur la position réelle.
  addEventListener("scroll", () => { if (!raf) target = current = scrollY; }, { passive: true });
  // Toute autre interaction interrompt l'animation en cours.
  ["mousedown", "keydown", "touchstart"].forEach(t => addEventListener(t, () => { cancelAnimationFrame(raf); raf = 0; }, { passive: true }));
}
