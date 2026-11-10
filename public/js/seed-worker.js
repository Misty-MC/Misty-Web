// Calcul du seed viewer dans le navigateur : cubiomes (MIT) compilé en WebAssembly.
// Chaque Worker a son propre module ; la page (seed-viewer.js) en lance plusieurs pour se
// servir de tous les cœurs du processeur du visiteur. Rien n'est calculé sur le serveur.
//
// Messages reçus : { id, op, … } ; réponse : { id, ok, result } ou { id, ok: false, error }.
//   init       { wasm }                                    → true
//   biomes     { v }                                      → { "<id>": ["nom", "#rrggbb"] }
//   spawn      { v, seed }                                → { x, z }
//   tile       { v, dim, seed, scale, x, z, size }        → ArrayBuffer (size² octets, un biome par case)
//   structures { v, dim, seed, x0, z0, x1, z1, types: [[clé, [types cubiomes]]] } → [[clé, x, z], …]
"use strict";

var e = null, ready = null;

// cubiomes n'utilise ces fonctions système que pour écrire des messages : on les neutralise
var wasi = {
  fd_close: function () { return 0; },
  fd_fdstat_get: function () { return 0; },
  fd_seek: function () { return 0; },
  fd_write: function () { return 0; },
  proc_exit: function (code) { throw new Error("cubiomes exit " + code); }
};

function load(url) {
  var imports = { wasi_snapshot_preview1: wasi };
  var p = WebAssembly.instantiateStreaming
    ? WebAssembly.instantiateStreaming(fetch(url), imports).catch(function () {
        // serveur sans le bon type MIME : on compile depuis les octets
        return fetch(url).then(function (r) { return r.arrayBuffer(); }).then(function (b) { return WebAssembly.instantiate(b, imports); });
      })
    : fetch(url).then(function (r) { return r.arrayBuffer(); }).then(function (b) { return WebAssembly.instantiate(b, imports); });
  return p.then(function (res) {
    e = res.instance.exports;
    e._initialize();
    return true;
  });
}

// Tampons réutilisés (la mémoire du module ne se libère pas vraiment)
var bufs = {};
function buf(name, size) {
  var b = bufs[name];
  if (!b || b.size < size) {
    if (b) e.misty_free(b.ptr);
    b = bufs[name] = { ptr: e.misty_alloc(size), size: size };
  }
  return b.ptr;
}

function cString(ptr) {
  var m = new Uint8Array(e.memory.buffer), end = ptr;
  while (m[end]) end++;
  return new TextDecoder().decode(m.subarray(ptr, end));
}

var ops = {
  biomes: function (m) {
    var p = buf("colors", 768);
    e.misty_couleurs(p);
    var c = new Uint8Array(e.memory.buffer, p, 768).slice(), out = {};
    for (var id = 0; id < 256; id++) {
      var n = e.misty_nom_biome(m.v, id);
      if (!n) continue;
      var hex = "#" + [c[id * 3], c[id * 3 + 1], c[id * 3 + 2]].map(function (x) { return (x < 16 ? "0" : "") + x.toString(16); }).join("");
      out[id] = [cString(n), hex];
    }
    return out;
  },
  spawn: function (m) {
    var p = buf("spawn", 8);
    e.misty_spawn(m.v, BigInt(m.seed), p, p + 4);
    var a = new Int32Array(e.memory.buffer, p, 2);
    return { x: a[0], z: a[1] };
  },
  tile: function (m) {
    var n = m.size * m.size, p = buf("tile", n);
    var err = e.misty_biomes(m.v, m.dim, BigInt(m.seed), m.scale, m.x * m.size, m.z * m.size, m.size, m.size, p);
    if (err) throw new Error("genBiomes " + err);
    return new Uint8Array(e.memory.buffer, p, n).slice().buffer;
  },
  structures: function (m) {
    var max = 256, p = buf("structs", max * 8), out = [], seed = BigInt(m.seed);
    m.types.forEach(function (t) {
      t[1].forEach(function (type) {
        var n = e.misty_structures(m.v, m.dim, seed, type, m.x0, m.z0, m.x1, m.z1, p, max);
        var a = new Int32Array(e.memory.buffer, p, n * 2);
        for (var i = 0; i < n; i++) out.push([t[0], a[i * 2], a[i * 2 + 1]]);
      });
    });
    return out;
  }
};

self.onmessage = function (ev) {
  var m = ev.data;
  if (m.op === "init") {
    ready = ready || load(m.wasm);
    ready.then(function () { self.postMessage({ id: m.id, ok: true, result: true }); },
      function (err) { self.postMessage({ id: m.id, ok: false, error: String(err) }); });
    return;
  }
  ready.then(function () {
    var result = ops[m.op](m);
    self.postMessage({ id: m.id, ok: true, result: result }, result instanceof ArrayBuffer ? [result] : []);
  }).catch(function (err) {
    self.postMessage({ id: m.id, ok: false, error: String(err) });
  });
};
