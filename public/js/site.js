// Petites améliorations côté navigateur. Tout le contenu est rendu par le serveur Rust :
// sans ce script, le site reste entièrement utilisable.
(() => {
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const HEADER_OFFSET = 84; // même valeur que scroll-padding-top dans base.css

  // Barre de navigation opaque dès qu'on a défilé
  const nav = document.querySelector(".nav");
  const onScroll = () => nav && nav.classList.toggle("scrolled", scrollY > 8);
  addEventListener("scroll", onScroll, { passive: true });
  onScroll();

  // Menu de langue : fermeture au clic à côté ou avec Échap
  const dd = document.querySelector(".langdd");
  if (dd) {
    document.addEventListener("click", e => { if (!dd.contains(e.target)) dd.open = false; });
    document.addEventListener("keydown", e => { if (e.key === "Escape") dd.open = false; });
  }

  // Patch notes : ouvrir la version visée par l'ancre (ex. /patch-notes#v0.3.1)
  const openFromHash = () => {
    const id = decodeURIComponent(location.hash.slice(1));
    const el = id && document.getElementById(id);
    if (el && el.tagName === "DETAILS") el.open = true;
  };
  openFromHash();
  addEventListener("hashchange", openFromHash);

  // Apparition au défilement
  const items = document.querySelectorAll(".reveal");
  if ("IntersectionObserver" in window && !reduce) {
    const io = new IntersectionObserver(entries => entries.forEach(e => {
      if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
    }), { threshold: 0.12 });
    items.forEach(el => io.observe(el));
  } else {
    items.forEach(el => el.classList.add("in"));
  }

  if (reduce) return;

  // Défilement fluide à la molette (inertie douce) et sur les liens d'ancre de la page
  let target = scrollY, current = scrollY, raf = 0;
  const clamp = v => Math.max(0, Math.min(document.documentElement.scrollHeight - innerHeight, v));
  const jump = y => scrollTo({ top: y, behavior: "instant" });
  const frame = () => {
    current += (target - current) * 0.1;
    if (Math.abs(target - current) < 0.5) { current = target; jump(current); raf = 0; return; }
    jump(current);
    raf = requestAnimationFrame(frame);
  };
  const animateTo = y => {
    if (!raf) current = scrollY;
    target = clamp(y);
    if (!raf) raf = requestAnimationFrame(frame);
  };
  const innerScrollable = (el, dy) => {
    for (; el && el !== document.body && el !== document.documentElement; el = el.parentElement) {
      const oy = getComputedStyle(el).overflowY;
      if ((oy === "auto" || oy === "scroll") && el.scrollHeight > el.clientHeight) {
        if (dy > 0 ? el.scrollTop + el.clientHeight < el.scrollHeight - 1 : el.scrollTop > 0) return true;
      }
    }
    return false;
  };

  addEventListener("click", e => {
    const a = e.target.closest("a[href*='#']");
    if (!a || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey) return;
    const url = new URL(a.href, location.href);
    if (url.pathname !== location.pathname || url.search !== location.search || !url.hash) return;
    const id = decodeURIComponent(url.hash.slice(1));
    const el = document.getElementById(id);
    if (!el && id !== "top") return;
    e.preventDefault();
    history.pushState(null, "", url.hash);
    if (el && el.tagName === "DETAILS") el.open = true;
    animateTo(id === "top" || !el ? 0 : el.getBoundingClientRect().top + scrollY - HEADER_OFFSET);
  });

  if (!matchMedia("(pointer: fine)").matches) return;
  addEventListener("wheel", e => {
    if (e.ctrlKey || e.defaultPrevented) return;
    let dy = e.deltaY;
    if (e.deltaMode === 1) dy *= 16; else if (e.deltaMode === 2) dy *= innerHeight;
    if (!dy || Math.abs(e.deltaX) > Math.abs(dy) || innerScrollable(e.target, dy)) return;
    e.preventDefault();
    animateTo((raf ? target : scrollY) + dy);
  }, { passive: false });
  addEventListener("scroll", () => { if (!raf) target = current = scrollY; }, { passive: true });
  ["mousedown", "keydown", "touchstart"].forEach(t =>
    addEventListener(t, () => { cancelAnimationFrame(raf); raf = 0; }, { passive: true }));
})();
