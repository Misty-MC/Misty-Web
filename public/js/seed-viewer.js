// Seed viewer (/tools/seed) : carte des biomes et structures, en tuiles calculées par le serveur
// (/api/seed/…). Chaque tuile est un tableau d'identifiants de biomes que l'on colore ici,
// ce qui permet aussi d'afficher le biome sous la souris.
(function () {
  "use strict";
  var app = document.getElementById("seed-app");
  if (!app) return;
  var cfg = JSON.parse(document.getElementById("seed-config").textContent);
  var T = cfg.tile, REGION = cfg.region, SCALES = cfg.scales;

  // Couleurs des structures (repères sur la carte et pastilles des filtres)
  var COLORS = {
    village: "#f0a23c", outpost: "#d24b4b", mansion: "#8a5a32", monument: "#2fc4c4", temple: "#f2dc5a",
    ruined_portal: "#a65ce6", ancient_city: "#2a6f8f", trial_chambers: "#9ee04a", fortress: "#e04040",
    bastion: "#7d7f8c", end_city: "#e07de0"
  };
  // Zoom : blocs par pixel (CSS), entre ces bornes
  var MIN_BPP = 0.25, MAX_BPP = 256;
  // Structures affichées seulement assez près (sinon trop de régions à demander)
  var STRUCT_MAX_BPP = 4;
  var MAX_REQ = 6;

  var $ = function (id) { return document.getElementById(id); };
  var canvas = $("seed-canvas"), ctx = canvas.getContext("2d"), box = $("seed-map");
  var input = $("seed-text"), versionSel = $("seed-version");
  var dimButtons = app.querySelectorAll(".seed-dims button");
  var chips = app.querySelectorAll("#seed-structs .chip");

  var state = { seed: null, text: "", v: cfg.versions[0][1], dim: 0, x: 0, z: 0, bpp: 4, spawn: null };
  var biomes = {}, palette = new Uint8Array(256 * 3), names = [];
  var tiles = new Map(), regions = new Map(), queue = [], inflight = 0, gen = 0;
  var hidden = {}, pointer = null;
  var W = 0, H = 0, dpr = 1, frame = 0;

  chips.forEach(function (c) {
    hidden[c.dataset.key] = !c.querySelector("input").checked;
    c.querySelector(".sw").style.setProperty("--c", COLORS[c.dataset.key] || "#888");
    c.querySelector("input").addEventListener("change", function (e) {
      hidden[c.dataset.key] = !e.target.checked;
      draw();
    });
  });

  // ---------- Données ----------
  function api(path, params) {
    var q = Object.keys(params).map(function (k) { return k + "=" + encodeURIComponent(params[k]); }).join("&");
    return "/api/seed/" + path + "?" + q;
  }

  function loadBiomes() {
    return fetch(api("biomes", { v: state.v })).then(function (r) { return r.json(); }).then(function (b) {
      biomes = b; palette = new Uint8Array(256 * 3); names = [];
      Object.keys(b).forEach(function (id) {
        var hex = b[id][1], i = +id;
        names[i] = b[id][0].replace(/_/g, " ").replace(/^./, function (c) { return c.toUpperCase(); });
        palette[i * 3] = parseInt(hex.substr(1, 2), 16);
        palette[i * 3 + 1] = parseInt(hex.substr(3, 2), 16);
        palette[i * 3 + 2] = parseInt(hex.substr(5, 2), 16);
      });
    });
  }

  function key(scale, tx, tz) { return gen + "|" + scale + "|" + tx + "|" + tz; }

  function request(scale, tx, tz) {
    var k = key(scale, tx, tz);
    if (tiles.has(k)) return;
    tiles.set(k, { state: "queued", scale: scale, tx: tx, tz: tz });
    queue.push(k);
  }

  function pump() {
    // les tuiles les plus proches du centre d'abord, et seulement celles encore utiles
    var cs = chooseScale(), view = visibleTiles(cs);
    queue = queue.filter(function (k) {
      var t = tiles.get(k);
      if (!t || t.state !== "queued") return false;
      var keep = t.scale === cs && t.tx >= view.x0 && t.tx <= view.x1 && t.tz >= view.z0 && t.tz <= view.z1;
      if (!keep) tiles.delete(k);
      return keep;
    });
    var span = T * cs;
    queue.sort(function (a, b) {
      var ta = tiles.get(a), tb = tiles.get(b);
      var da = Math.hypot((ta.tx + .5) * span - state.x, (ta.tz + .5) * span - state.z);
      var db = Math.hypot((tb.tx + .5) * span - state.x, (tb.tz + .5) * span - state.z);
      return da - db;
    });
    while (inflight < MAX_REQ && queue.length) fetchTile(queue.shift());
  }

  function fetchTile(k) {
    var t = tiles.get(k), myGen = gen;
    t.state = "loading"; inflight++;
    fetch(api("tile", { seed: state.seed, v: state.v, dim: state.dim, scale: t.scale, x: t.tx, z: t.tz }))
      .then(function (r) { if (!r.ok) throw r.status; return r.arrayBuffer(); })
      .then(function (buf) {
        if (myGen !== gen) return;
        var ids = new Uint8Array(buf), img = new ImageData(T, T), d = img.data;
        for (var i = 0; i < ids.length; i++) {
          var p = ids[i] * 3;
          d[i * 4] = palette[p]; d[i * 4 + 1] = palette[p + 1]; d[i * 4 + 2] = palette[p + 2]; d[i * 4 + 3] = 255;
        }
        var c = document.createElement("canvas");
        c.width = c.height = T;
        c.getContext("2d").putImageData(img, 0, 0);
        t.ids = ids; t.canvas = c; t.state = "ready";
        note(null);
      })
      .catch(function (status) {
        if (myGen !== gen) return;
        tiles.delete(k);
        if (status === 429) setTimeout(draw, 3000); else note(cfg.text.error);
      })
      .finally(function () { inflight--; if (myGen === gen) { draw(); } });
  }

  function requestRegion(rx, rz) {
    var k = gen + "|" + rx + "|" + rz;
    if (regions.has(k)) return;
    var myGen = gen;
    regions.set(k, null);
    fetch(api("structures", { seed: state.seed, v: state.v, dim: state.dim, x: rx, z: rz }))
      .then(function (r) { if (!r.ok) throw r.status; return r.json(); })
      .then(function (list) { if (myGen === gen) { regions.set(k, list); draw(); } })
      .catch(function () { if (myGen === gen) regions.delete(k); });
  }

  // ---------- Géométrie ----------
  function chooseScale() {
    // une case fait au moins 2 pixels à l'écran : assez fin sans trop de tuiles
    for (var i = 0; i < SCALES.length; i++) if (SCALES[i] >= state.bpp * 2) return SCALES[i];
    return SCALES[SCALES.length - 1];
  }
  function toWorld(px, py) { return { x: state.x + (px - W / 2) * state.bpp, z: state.z + (py - H / 2) * state.bpp }; }
  function toScreen(bx, bz) { return { x: (bx - state.x) / state.bpp + W / 2, y: (bz - state.z) / state.bpp + H / 2 }; }
  function visibleTiles(scale) {
    var span = T * scale, a = toWorld(0, 0), b = toWorld(W, H);
    return { x0: Math.floor(a.x / span), z0: Math.floor(a.z / span), x1: Math.floor(b.x / span), z1: Math.floor(b.z / span) };
  }

  // ---------- Dessin ----------
  function draw() {
    if (!frame) frame = requestAnimationFrame(render);
  }

  function render() {
    frame = 0;
    if (state.seed === null) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = "#05080f";
    ctx.fillRect(0, 0, W, H);
    ctx.imageSmoothingEnabled = false;

    var cs = chooseScale();
    // les tuiles plus grossières déjà chargées servent d'aperçu en attendant les fines
    for (var i = SCALES.length - 1; i >= 0; i--) {
      var s = SCALES[i];
      if (s < cs) break;
      var v = visibleTiles(s), span = T * s;
      for (var tz = v.z0; tz <= v.z1; tz++) for (var tx = v.x0; tx <= v.x1; tx++) {
        var t = tiles.get(key(s, tx, tz));
        if (s === cs && !t) request(s, tx, tz);
        if (t && t.state === "ready") {
          var p = toScreen(tx * span, tz * span), size = span / state.bpp;
          ctx.drawImage(t.canvas, Math.floor(p.x), Math.floor(p.y), Math.ceil(size + (p.x % 1)) + 1, Math.ceil(size + (p.y % 1)) + 1);
        }
      }
    }
    pump();
    drawAxes();
    drawStructures();
    drawSpawn();
    if (tiles.size > 3000) pruneTiles(cs);
  }

  function drawAxes() {
    var o = toScreen(0, 0);
    ctx.strokeStyle = "rgba(255,255,255,.35)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    if (o.x >= 0 && o.x <= W) { ctx.moveTo(Math.round(o.x) + .5, 0); ctx.lineTo(Math.round(o.x) + .5, H); }
    if (o.y >= 0 && o.y <= H) { ctx.moveTo(0, Math.round(o.y) + .5); ctx.lineTo(W, Math.round(o.y) + .5); }
    ctx.stroke();
  }

  var visibleStructs = [];
  function drawStructures() {
    visibleStructs = [];
    var wanted = Array.prototype.some.call(chips, function (c) { return !c.hidden && !hidden[c.dataset.key]; });
    structMsg = wanted && state.bpp > STRUCT_MAX_BPP ? cfg.text.zoomMore : "";
    showNote();
    if (state.bpp > STRUCT_MAX_BPP) return;
    var a = toWorld(0, 0), b = toWorld(W, H);
    for (var rz = Math.floor(a.z / REGION); rz <= Math.floor(b.z / REGION); rz++)
      for (var rx = Math.floor(a.x / REGION); rx <= Math.floor(b.x / REGION); rx++) {
        var list = regions.get(gen + "|" + rx + "|" + rz);
        if (list === undefined) { requestRegion(rx, rz); continue; }
        if (!list) continue;
        list.forEach(function (s) {
          if (hidden[s[0]]) return;
          var p = toScreen(s[1], s[2]);
          if (p.x < -10 || p.y < -10 || p.x > W + 10 || p.y > H + 10) return;
          visibleStructs.push({ k: s[0], x: s[1], z: s[2], px: p.x, py: p.y });
          marker(p.x, p.y, COLORS[s[0]] || "#fff");
        });
      }
  }

  function marker(x, y, color) {
    var r = state.bpp <= 2 ? 7 : 5.5;
    ctx.fillStyle = "rgba(5,8,15,.85)";
    ctx.fillRect(Math.round(x - r - 1.5), Math.round(y - r - 1.5), Math.round(2 * r + 3), Math.round(2 * r + 3));
    ctx.fillStyle = "#fff";
    ctx.fillRect(Math.round(x - r), Math.round(y - r), Math.round(2 * r), Math.round(2 * r));
    ctx.fillStyle = color;
    ctx.fillRect(Math.round(x - r + 1.5), Math.round(y - r + 1.5), Math.round(2 * r - 3), Math.round(2 * r - 3));
  }

  function drawSpawn() {
    if (!state.spawn || state.dim !== 0) return;
    var p = toScreen(state.spawn.x, state.spawn.z);
    ctx.save();
    ctx.translate(p.x, p.y);
    ctx.fillStyle = "#ff4d5e"; ctx.strokeStyle = "#05080f"; ctx.lineWidth = 2;
    ctx.beginPath();
    for (var i = 0; i < 10; i++) {
      var r = i % 2 ? 4 : 10, a = -Math.PI / 2 + i * Math.PI / 5;
      ctx.lineTo(Math.cos(a) * r, Math.sin(a) * r);
    }
    ctx.closePath(); ctx.stroke(); ctx.fill();
    ctx.restore();
  }

  function pruneTiles(cs) {
    var v = visibleTiles(cs);
    tiles.forEach(function (t, k) {
      if (t.state !== "ready") return;
      if (t.scale !== cs || t.tx < v.x0 - 4 || t.tx > v.x1 + 4 || t.tz < v.z0 - 4 || t.tz > v.z1 + 4) tiles.delete(k);
    });
  }

  // Bandeau : une erreur de calcul passe avant le conseil de zoomer
  var noteEl = $("seed-note"), errorMsg = "", structMsg = "";
  function note(text) { errorMsg = text || ""; showNote(); }
  function showNote() {
    var text = errorMsg || structMsg;
    noteEl.hidden = !text;
    noteEl.textContent = text;
  }

  // ---------- Infos sous le curseur ----------
  function biomeAt(bx, bz) {
    for (var i = 0; i < SCALES.length; i++) {
      var s = SCALES[i], span = T * s;
      if (s < chooseScale()) continue;
      var tx = Math.floor(bx / span), tz = Math.floor(bz / span);
      var t = tiles.get(key(s, tx, tz));
      if (t && t.state === "ready") {
        var cx = Math.floor((bx - tx * span) / s), cz = Math.floor((bz - tz * span) / s);
        return names[t.ids[cz * T + cx]] || "?";
      }
    }
    return "–";
  }

  function hover(px, py) {
    var w = toWorld(px, py), bx = Math.floor(w.x), bz = Math.floor(w.z);
    var text = biomeAt(bx, bz), near = null;
    visibleStructs.forEach(function (s) {
      var d = Math.hypot(s.px - px, s.py - py);
      if (d < 10 && (!near || d < near.d)) near = { d: d, s: s };
    });
    if (near) { text = cfg.labels[near.s.k] + " · " + near.s.x + ", " + near.s.z; bx = near.s.x; bz = near.s.z; }
    else if (state.spawn && state.dim === 0) {
      var sp = toScreen(state.spawn.x, state.spawn.z);
      if (Math.hypot(sp.x - px, sp.y - py) < 12) text = cfg.text.spawn + " · " + state.spawn.x + ", " + state.spawn.z;
    }
    $("seed-biome").textContent = text;
    $("seed-coords").textContent = "X " + bx + " · Z " + bz;
  }

  // ---------- Interactions ----------
  function zoomAt(factor, px, py) {
    var before = toWorld(px, py);
    state.bpp = Math.min(MAX_BPP, Math.max(MIN_BPP, state.bpp * factor));
    var after = toWorld(px, py);
    state.x += before.x - after.x; state.z += before.z - after.z;
    changed();
  }

  var pointers = new Map(), pinch = null;
  box.addEventListener("pointerdown", function (e) {
    if (e.target.closest(".seed-tools")) return;
    box.setPointerCapture(e.pointerId);
    pointers.set(e.pointerId, { x: e.offsetX, y: e.offsetY });
    if (pointers.size === 2) {
      var p = Array.from(pointers.values());
      pinch = { d: Math.hypot(p[0].x - p[1].x, p[0].y - p[1].y) };
    }
    box.classList.add("drag");
  });
  box.addEventListener("pointermove", function (e) {
    var prev = pointers.get(e.pointerId);
    if (!prev) { hover(e.offsetX, e.offsetY); return; }
    var cur = { x: e.offsetX, y: e.offsetY };
    pointers.set(e.pointerId, cur);
    if (pointers.size === 2 && pinch) {
      var p = Array.from(pointers.values()), d = Math.hypot(p[0].x - p[1].x, p[0].y - p[1].y);
      if (d > 0 && pinch.d > 0) zoomAt(pinch.d / d, (p[0].x + p[1].x) / 2, (p[0].y + p[1].y) / 2);
      pinch.d = d;
    } else if (pointers.size === 1) {
      state.x -= (cur.x - prev.x) * state.bpp; state.z -= (cur.y - prev.y) * state.bpp;
      changed();
    }
    hover(e.offsetX, e.offsetY);
  });
  function release(e) {
    pointers.delete(e.pointerId);
    if (pointers.size < 2) pinch = null;
    if (!pointers.size) box.classList.remove("drag");
  }
  box.addEventListener("pointerup", release);
  box.addEventListener("pointercancel", release);
  box.addEventListener("wheel", function (e) {
    e.preventDefault();
    zoomAt(Math.pow(1.0018, e.deltaY * (e.deltaMode === 1 ? 30 : 1)), e.offsetX, e.offsetY);
  }, { passive: false });

  $("seed-zoom-in").onclick = function () { zoomAt(1 / 1.6, W / 2, H / 2); };
  $("seed-zoom-out").onclick = function () { zoomAt(1.6, W / 2, H / 2); };
  $("seed-go-origin").onclick = function () { state.x = 0; state.z = 0; changed(); };
  $("seed-go-spawn").onclick = function () {
    if (state.spawn) { state.x = state.spawn.x; state.z = state.spawn.z; changed(); }
  };
  $("seed-share").onclick = function () {
    saveHash();
    var b = this, label = b.textContent;
    var done = function () { b.textContent = cfg.text.copied; setTimeout(function () { b.textContent = label; }, 1500); };
    if (navigator.clipboard) navigator.clipboard.writeText(location.href).then(done, done); else done();
  };

  $("seed-form").addEventListener("submit", function (e) { e.preventDefault(); open(input.value, true); });
  $("seed-random").onclick = function () { input.value = ""; open("", true); };
  versionSel.addEventListener("change", function () {
    state.v = +versionSel.value;
    updateDims();
    loadBiomes().then(function () { open(state.seed, false); });
  });
  dimButtons.forEach(function (b) {
    b.addEventListener("click", function () {
      if (b.disabled) return;
      state.dim = +b.dataset.dim;
      updateDims();
      reset();
    });
  });

  function updateDims() {
    // Nether depuis la 1.16 (19), End depuis la 1.9 (12), comme côté serveur
    dimButtons.forEach(function (b) {
      var d = +b.dataset.dim;
      b.disabled = (d === -1 && state.v < 19) || (d === 1 && state.v < 12);
      if (b.disabled && state.dim === d) state.dim = 0;
    });
    dimButtons.forEach(function (b) { b.setAttribute("aria-pressed", String(+b.dataset.dim === state.dim)); });
    chips.forEach(function (c) {
      var dims = cfg.structures.filter(function (s) { return s[0] === c.dataset.key; }).map(function (s) { return s[1]; });
      c.hidden = dims.indexOf(state.dim) < 0;
    });
  }

  function reset() {
    gen++;
    tiles.clear(); regions.clear(); queue = [];
    changed();
  }

  var hashTimer = 0;
  function changed() {
    draw();
    clearTimeout(hashTimer);
    hashTimer = setTimeout(saveHash, 400);
  }

  function saveHash() {
    if (state.seed === null) return;
    var h = "#seed=" + encodeURIComponent(state.seed) + "&v=" + state.v + "&dim=" + state.dim +
      "&x=" + Math.round(state.x) + "&z=" + Math.round(state.z) + "&zoom=" + +state.bpp.toFixed(3);
    history.replaceState(null, "", h);
  }

  // Ouvre une seed (texte ou nombre). `recentrer` : se placer sur le point d'apparition.
  function open(text, recentrer) {
    return fetch(api("info", { seed: text, v: state.v }))
      .then(function (r) { if (!r.ok) throw r.status; return r.json(); })
      .then(function (info) {
        state.seed = info.seed;
        state.spawn = info.spawn;
        if (!input.value.trim() || recentrer) input.value = text.trim() ? text.trim() : info.seed;
        if (recentrer) { state.x = info.spawn.x; state.z = info.spawn.z; }
        reset();
      })
      .catch(function () { note(cfg.text.error); });
  }

  function resize() {
    var r = box.getBoundingClientRect();
    dpr = window.devicePixelRatio || 1;
    W = r.width; H = r.height;
    canvas.width = Math.round(W * dpr); canvas.height = Math.round(H * dpr);
    draw();
  }
  new ResizeObserver(resize).observe(box);

  // Démarrage : la carte du lien partagé, sinon une seed au hasard
  var h = {};
  location.hash.slice(1).split("&").forEach(function (p) {
    var i = p.indexOf("=");
    if (i > 0) h[p.slice(0, i)] = decodeURIComponent(p.slice(i + 1));
  });
  if (h.v && cfg.versions.some(function (v) { return v[1] === +h.v; })) state.v = +h.v;
  versionSel.value = state.v;
  if (h.dim) state.dim = +h.dim || 0;
  updateDims();
  var hasPos = h.x !== undefined && h.z !== undefined && isFinite(+h.x) && isFinite(+h.z);
  if (hasPos) { state.x = +h.x; state.z = +h.z; }
  if (h.zoom && isFinite(+h.zoom)) state.bpp = Math.min(MAX_BPP, Math.max(MIN_BPP, +h.zoom));
  loadBiomes().then(function () {
    input.value = h.seed || "";
    open(h.seed || "", !hasPos);
  });
})();
