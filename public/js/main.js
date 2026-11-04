// Point d'entrée du site.
import { initLang, onLangChange } from "./lang.js";
import { loadRelease, renderRelease } from "./release.js";
import { initScrollEffects, initMarquee, initShowcase, initReveal, initFooterYear } from "./ui.js";

onLangChange(renderRelease);
initLang();
loadRelease();

initMarquee();
initScrollEffects();
initShowcase();
initReveal();
initFooterYear();
