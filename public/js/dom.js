// Petits raccourcis DOM partagés par les modules.
export const $ = id => document.getElementById(id);
export const $$ = sel => document.querySelectorAll(sel);
export const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;
