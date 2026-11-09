// Vérification dans Discord (Activité) : le SDK Embedded App dit qui est le membre et dans quel
// serveur il est ; le captcha se fait ensuite ici, sans changer de page.
import { DiscordSDK } from "/js/discord-sdk.js";

const app = document.getElementById("vf-app");
const $ = id => document.getElementById(id);
const d = app.dataset;
let sdk = null;
let token = "";
let captcha = "";

const post = (url, body) =>
  fetch(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) })
    .then(r => r.json())
    .catch(() => ({ error: "network" }));

function end(ok, title, text) {
  $("vf-wait").hidden = true;
  $("vf-form").hidden = true;
  $("vf-end").hidden = false;
  $("vf-end").className = "vf-end " + (ok ? "ok" : "bad");
  $("vf-badge").className = "vf-badge " + (ok ? "ok" : "bad");
  $("vf-badge").textContent = ok ? "✓" : "!";
  $("vf-title").textContent = title;
  $("vf-text").textContent = text;
}

function fail(error) {
  const map = {
    off: [d.offTitle, d.off], guild: [d.offTitle, d.off], many: [d.badTitle, d.many],
    role: [d.roleTitle, d.role], auth: [d.badTitle, d.auth], expired: [d.badTitle, d.bad],
  };
  const [title, text] = map[error] || [d.badTitle, d.auth];
  end(false, title, text);
}

function showCaptcha(id, wrong) {
  captcha = id;
  $("vf-img").src = `/verify/captcha/${id}.svg`;
  $("vf-wait").hidden = true;
  $("vf-form").hidden = false;
  $("vf-err").hidden = !wrong;
  $("vf-err").textContent = d.wrong;
  $("vf-code").value = "";
  $("vf-code").focus();
}

$("vf-form").addEventListener("submit", async e => {
  e.preventDefault();
  const r = await post("/activity/answer", { t: token, id: captcha, code: $("vf-code").value });
  if (r.ok) end(true, $("vf-ok").dataset.title, $("vf-ok").dataset.text);
  else if (r.captcha) showCaptcha(r.captcha, true);
  else fail(r.error);
});

$("vf-new").addEventListener("click", async () => {
  const r = await post("/activity/new", { t: token });
  if (r.captcha) showCaptcha(r.captcha, false);
  else fail(r.error);
});

$("vf-close").addEventListener("click", () => {
  if (sdk) sdk.close(1000, "done");
});

(async () => {
  if (!d.client) return fail("off");
  try {
    sdk = new DiscordSDK(d.client);
    await sdk.ready();
    const { code } = await sdk.commands.authorize({
      client_id: d.client,
      response_type: "code",
      state: "",
      prompt: "none",
      scope: ["identify"],
    });
    const r = await post("/activity/start", { code, guild_id: sdk.guildId });
    if (r.token && r.captcha) {
      token = r.token;
      showCaptcha(r.captcha, false);
    } else fail(r.error);
  } catch (err) {
    console.error(err);
    fail("auth");
  }
})();
