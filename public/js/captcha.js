// Captcha puzzle : faire glisser la pièce jusqu'au trou. Partagé par /verify (navigateur) et
// /activity (Discord). Les images viennent du serveur ; seule la position finale et le geste sont
// envoyés, le serveur décide.
const $ = id => document.getElementById(id);

export function captcha({ onDone } = {}) {
  const root = $("cp");
  const d = root.dataset;
  const W = +d.w, H = +d.h, PW = +d.pieceW;
  let token = "", id = "", busy = false;

  const post = (url, body) =>
    fetch(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) })
      .then(r => r.json())
      .catch(() => ({ error: "network" }));

  function steps(n) {
    [1, 2, 3].forEach(i => {
      const li = $("cp-s" + i);
      li.classList.toggle("done", i < n);
      li.classList.toggle("on", i === n);
    });
  }

  // ---- Glissière ----
  const slider = $("cp-slider"), knob = $("cp-knob"), fill = $("cp-fill"), piece = $("cp-piece");
  let pos = 0, t0 = 0, trace = [];
  const track = () => Math.max(1, slider.clientWidth - knob.offsetWidth);
  // position de la pièce dans l'image : de START (bien visible au départ) au bord droit
  const START = 6;
  const xImage = () => START + (pos / track()) * (W - PW - START);
  function place() {
    knob.style.transform = `translateX(${pos}px)`;
    fill.style.width = `${pos + knob.offsetWidth / 2}px`;
    piece.style.left = `${((xImage() - 3) / W) * 100}%`;
  }
  function reset() {
    pos = 0;
    trace = [];
    slider.classList.remove("moving");
    place();
  }
  knob.addEventListener("pointerdown", e => {
    if (busy) return;
    knob.setPointerCapture(e.pointerId);
    const start = e.clientX - pos;
    t0 = performance.now();
    trace = [[0, xImage()]];
    slider.classList.add("moving");
    const move = ev => {
      pos = Math.min(track(), Math.max(0, ev.clientX - start));
      trace.push([Math.round(performance.now() - t0), +xImage().toFixed(1)]);
      place();
    };
    const up = () => {
      knob.removeEventListener("pointermove", move);
      knob.removeEventListener("pointerup", up);
      knob.removeEventListener("pointercancel", up);
      slider.classList.remove("moving");
      if (pos < 4) return reset();
      submit();
    };
    knob.addEventListener("pointermove", move);
    knob.addEventListener("pointerup", up);
    knob.addEventListener("pointercancel", up);
  });

  function load(captchaId, y) {
    id = captchaId;
    $("cp-bg").src = `/verify/captcha/${id}/bg.png`;
    piece.src = `/verify/captcha/${id}/piece.png`;
    piece.style.top = `${((y - 3) / H) * 100}%`;
    piece.style.width = `${((PW + 6) / W) * 100}%`;
    reset();
  }

  async function submit() {
    busy = true;
    slider.classList.add("busy");
    const r = await post("/verify/answer", { t: token, id, x: +xImage().toFixed(1), trace });
    busy = false;
    slider.classList.remove("busy");
    if (r.ok) return done();
    if (r.captcha) {
      $("cp-err").textContent = d.wrong;
      $("cp-err").hidden = false;
      slider.classList.add("shake");
      setTimeout(() => slider.classList.remove("shake"), 400);
      return load(r.captcha, r.y);
    }
    fail(r.error);
  }

  $("cp-another").addEventListener("click", async () => {
    if (busy) return;
    const r = await post("/verify/new", { t: token });
    if (r.captcha) {
      $("cp-err").hidden = true;
      load(r.captcha, r.y);
    } else fail(r.error);
  });

  $("cp-start").addEventListener("click", () => {
    $("cp-consent").hidden = true;
    $("cp-challenge").hidden = false;
    steps(2);
    place();
  });

  function end(ok, title, text) {
    $("cp-consent").hidden = true;
    $("cp-challenge").hidden = true;
    $("cp-end").hidden = false;
    $("cp-end").className = "vf-end " + (ok ? "ok" : "bad");
    $("cp-badge").className = "vf-badge " + (ok ? "ok" : "bad");
    $("cp-badge").textContent = ok ? "✓" : "!";
    $("cp-title").textContent = title;
    $("cp-msg").textContent = text;
  }

  function done() {
    steps(3);
    $("cp-s3").classList.add("done");
    end(true, d.okTitle, d.ok);
    if (onDone) onDone();
  }

  function fail(error, detail) {
    const map = {
      off: [d.offTitle, d.off], guild: [d.offTitle, d.off], many: [d.badTitle, d.many],
      role: [d.roleTitle, d.role], auth: [d.authTitle, d.auth], expired: [d.badTitle, d.bad],
    };
    const [title, text] = map[error] || [d.authTitle, d.auth];
    end(false, title, text);
    $("cp-detail").textContent = detail || "";
    $("cp-detail").hidden = !detail;
  }

  return {
    // Lien et premier puzzle prêts : le bouton « Commencer » s'active.
    ready(t, captchaId, y) {
      token = t;
      load(captchaId, y);
      $("cp-start-label").textContent = d.start;
      $("cp-start").disabled = false;
    },
    fail,
    // Fermeture automatique (Activité) : compte à rebours puis `close()`.
    autoClose(seconds, close) {
      const label = $("cp-auto");
      $("cp-close").hidden = false;
      $("cp-close").onclick = close;
      const tick = () => {
        label.textContent = d.autoclose.replace("{0}", seconds);
        label.hidden = false;
        if (seconds-- <= 0) close();
        else setTimeout(tick, 1000);
      };
      tick();
    },
  };
}
