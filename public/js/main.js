// Point d'entrée du site.
import { initLang, onLangChange } from "./lang.js";
import { loadRelease, renderRelease } from "./release.js";
import { initSmoothScroll } from "./smooth-scroll.js";
import { initScrollEffects, initMarquee, initShowcase, initReveal, initFooterYear } from "./ui.js";

onLangChange(renderRelease);
initLang();
loadRelease();

initSmoothScroll();
initMarquee();
initScrollEffects();
initShowcase();
initReveal();
initFooterYear();
