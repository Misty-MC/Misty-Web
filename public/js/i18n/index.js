// Point d'entrée des traductions : une langue = un fichier.
import fr from "./fr.js";
import en from "./en.js";
import es from "./es.js";
import ru from "./ru.js";
import it from "./it.js";

export const TRANSLATIONS = { fr, en, es, ru, it };
export const LANGS = Object.keys(TRANSLATIONS);
