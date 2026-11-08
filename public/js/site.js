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

  // Menus déroulants (langue, menu mobile) : fermeture au clic à côté ou avec Échap
  const menus = [...document.querySelectorAll("details.langdd, details.burger")];
  document.addEventListener("click", e => menus.forEach(m => { if (!m.contains(e.target)) m.open = false; }));
  document.addEventListener("keydown", e => { if (e.key === "Escape") menus.forEach(m => { m.open = false; }); });
  // Un lien choisi dans un menu le referme
  menus.forEach(m => m.querySelectorAll("a").forEach(a => a.addEventListener("click", () => { m.open = false; })));
  // Un seul menu ouvert à la fois
  menus.forEach(m => m.addEventListener("toggle", () => { if (m.open) menus.forEach(o => { if (o !== m) o.open = false; }); }));

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

  // Cliquer sur la page où l'on est déjà (lien de la barre, logo…) : on remonte en haut
  // en douceur au lieu de recharger la page.
  document.addEventListener("click", e => {
    const a = e.target.closest("a[href]");
    if (!a || e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || a.target === "_blank") return;
    const url = new URL(a.href, location.href);
    if (url.origin !== location.origin || url.pathname !== location.pathname || url.search || url.hash) return;
    e.preventDefault();
    if (location.hash) history.replaceState(null, "", location.pathname + location.search);
    scrollTo({ top: 0, behavior: reduce ? "auto" : "smooth" });
  });

  // ---------- Boutique : vitrine 3D du skin ----------
  const stage = document.getElementById("stage");
  if (stage && window.skinview3d) {
    const sv = window.skinview3d;
    const canvas = document.getElementById("skin3d");
    const viewer = new sv.SkinViewer({ canvas, width: canvas.clientWidth || 360, height: canvas.clientHeight || 420, zoom: 0.62, fov: 36 });
    viewer.globalLight.intensity = 2.4;
    viewer.cameraLight.intensity = 0.8;
    viewer.controls.enableZoom = false;
    viewer.controls.enablePan = false;
    viewer.autoRotate = !reduce;
    viewer.autoRotateSpeed = 0.7;
    viewer.animation = new sv.WalkingAnimation();
    viewer.animation.speed = 0.55;
    viewer.animation.paused = reduce;

    const setName = name => {
      viewer.nameTag = name
        ? new sv.NameTagObject(name, { font: "48px Silkscreen", height: 2.4, margin: [4, 8, 2, 8], textStyle: "#eaf1ff", backgroundStyle: "rgba(10,15,29,.55)" })
        : null;
    };
    const loadSkin = url => viewer.loadSkin(url).catch(() => viewer.loadSkin("/skin/MHF_Steve").catch(() => {}));
    loadSkin(stage.dataset.skin);
    setName(stage.dataset.name);

    // Cape et élytres « Misty », dessinées en pixel art : dégradé nuit → brume et petit nuage.
    const mistyTexture = () => {
      const c = document.createElement("canvas");
      c.width = 64; c.height = 32;
      const g = c.getContext("2d");
      const shade = (x, y, h, x0) => {
        const t = y / h, d = ((x + y) % 2) * 6 - 3 + (((x - x0) * 7 + y * 3) % 5) - 2;
        const r = Math.round(26 + 70 * t + d), gg = Math.round(40 + 105 * t + d), b = Math.round(77 + 160 * t + d);
        return `rgb(${r},${gg},${b})`;
      };
      const fill = (x0, y0, w, h) => {
        for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) { g.fillStyle = shade(x, y - y0, h, x0); g.fillRect(x, y, 1, 1); }
      };
      const cloud = (ox, oy) => {
        const rows = ["..###...", ".######.", "########", "########", ".######."];
        rows.forEach((row, y) => [...row].forEach((ch, x) => {
          if (ch !== "#") return;
          g.fillStyle = y === 4 ? "#b9cdf5" : "#eaf1ff"; g.fillRect(ox + x, oy + y, 1, 1);
        }));
        g.fillStyle = "#cfe0ff"; g.fillRect(ox, oy + 6, 6, 1); g.fillRect(ox + 4, oy + 8, 4, 1);
      };
      fill(0, 0, 22, 17);            // cape (toutes les faces)
      cloud(2, 5); cloud(12, 5);     // nuage dos et face
      fill(22, 0, 24, 22);           // élytres
      g.fillStyle = "rgba(10,15,29,.35)";
      for (let x = 24; x < 46; x += 3) g.fillRect(x, 2, 1, 18); // plumes
      cloud(34, 6);
      return c;
    };
    const texture = mistyTexture();
    stage.querySelectorAll(".try button").forEach(btn => btn.addEventListener("click", () => {
      const back = btn.dataset.back;
      viewer.loadCape(back === "none" ? null : texture, back === "none" ? undefined : { backEquipment: back });
      stage.querySelectorAll(".try button").forEach(b => b.setAttribute("aria-pressed", String(b === btn)));
    }));

    // Taille du rendu = taille affichée ; pas de rendu quand la vitrine est hors de l'écran.
    const resize = () => { viewer.width = canvas.clientWidth; viewer.height = canvas.clientHeight; };
    if ("ResizeObserver" in window) new ResizeObserver(resize).observe(canvas);
    if ("IntersectionObserver" in window) {
      new IntersectionObserver(([e]) => { viewer.renderPaused = !e.isIntersecting; }).observe(stage);
    }

    // Aperçu en direct pendant qu'on tape son pseudo
    const input = document.getElementById("player");
    let timer;
    input?.addEventListener("input", () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const name = input.value.trim();
        if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) return;
        loadSkin(`/skin/${name}`);
        setName(name);
      }, 450);
    });
    stage.classList.add("ready");
  }

  // ---------- Documentation ----------
  // Recherche instantanée : filtre les pages sur leur titre, description et intertitres,
  // puis sur tout leur texte dès que l'index plein texte est chargé (au premier caractère tapé).
  let fullText = null, loading = null;
  const loadIndex = () => loading ??= fetch(`/docs/search.json?lang=${document.documentElement.lang}`)
    .then(r => r.json())
    .then(list => { fullText = new Map(list.map(e => [e.slug, e.text])); })
    .catch(() => { fullText = new Map(); });
  document.querySelectorAll(".docs-search input").forEach(input => {
    const scope = input.closest(".docs-side") || input.closest(".docs-hero")?.parentElement || document;
    const filter = () => {
      // Chaque mot doit commencer un mot du texte (« ram » trouve « RAM », pas « programme »).
      const words = input.value.trim().toLowerCase().split(/\s+/).filter(Boolean)
        .map(w => new RegExp(`(^|[^\\p{L}\\p{N}])${w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`, "u"));
      scope.querySelectorAll(".docs-nav, .docs-map").forEach(list => {
        let shown = 0;
        list.querySelectorAll("li[data-search]").forEach(li => {
          const body = fullText?.get(li.dataset.slug) ?? "";
          const ok = words.every(w => w.test(li.dataset.search) || w.test(body));
          li.hidden = !ok;
          if (ok) shown++;
        });
        list.querySelectorAll(".nav-sec, .map-sec").forEach(sec => {
          sec.hidden = !sec.querySelector("li[data-search]:not([hidden])");
        });
        const empty = list.querySelector(".docs-empty");
        if (empty) empty.hidden = shown > 0;
      });
    };
    input.addEventListener("input", () => {
      filter();
      if (!fullText) loadIndex().then(filter);
    });
    input.addEventListener("focus", () => { loadIndex(); }, { once: true });
    input.addEventListener("keydown", e => {
      if (e.key === "Enter") {
        const first = scope.querySelector("li[data-search]:not([hidden]) a");
        if (first) location.href = first.href;
      } else if (e.key === "Escape") {
        input.value = ""; filter(); input.blur();
      }
    });
  });
  // « / » place le curseur dans la recherche
  document.addEventListener("keydown", e => {
    if (e.key !== "/" || e.ctrlKey || e.metaKey || /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement.tagName)) return;
    const input = [...document.querySelectorAll(".docs-search input")].find(i => i.offsetParent);
    if (input) { e.preventDefault(); input.focus(); }
  });

  // Bouton « copier » sur les blocs de code
  document.querySelectorAll(".md-doc pre").forEach(pre => {
    const btn = document.createElement("button");
    btn.type = "button"; btn.className = "copy"; btn.textContent = "Copy";
    btn.addEventListener("click", () => {
      navigator.clipboard?.writeText(pre.querySelector("code")?.innerText ?? pre.innerText).then(() => {
        btn.textContent = "✓"; btn.classList.add("done");
        setTimeout(() => { btn.textContent = "Copy"; btn.classList.remove("done"); }, 1500);
      });
    });
    pre.appendChild(btn);
  });

  // Sommaire : met en avant la section en cours de lecture
  const tocLinks = [...document.querySelectorAll(".docs-toc a")];
  if (tocLinks.length && "IntersectionObserver" in window) {
    const byId = new Map(tocLinks.map(a => [decodeURIComponent(a.hash.slice(1)), a]));
    const visible = new Set();
    const spy = new IntersectionObserver(entries => {
      entries.forEach(e => e.isIntersecting ? visible.add(e.target.id) : visible.delete(e.target.id));
      const current = [...byId.keys()].find(id => visible.has(id));
      if (current) tocLinks.forEach(a => a.classList.toggle("active", a === byId.get(current)));
    }, { rootMargin: "-80px 0px -60% 0px" });
    byId.forEach((_, id) => { const h = document.getElementById(id); if (h) spy.observe(h); });
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
