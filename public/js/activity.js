// Vérification dans Discord (Activité) : le SDK Embedded App dit qui est le membre et dans quel
// serveur il est ; le puzzle se fait ensuite comme dans le navigateur.
import { DiscordSDK } from "/js/discord-sdk.js";
import { captcha } from "/js/captcha.js";

const client = document.getElementById("cp-data").dataset.client;
let sdk = null;
const close = () => sdk && sdk.close(1000, "done");
const cp = captcha({ onDone: () => cp.autoClose(10, close) });

const post = (url, body) =>
  fetch(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) })
    .then(r => r.json())
    .catch(() => ({ error: "network" }));

// Erreur côté Discord : notée dans les logs du site, pour savoir ce qui bloque.
function report(stage, err) {
  const detail = `${stage}: ${err && (err.code !== undefined ? err.code + " " : "")}${(err && err.message) || err}`;
  post("/activity/log", { detail }).catch(() => {});
  return detail;
}

(async () => {
  if (!client) return cp.fail("off");
  let stage = "sdk";
  try {
    sdk = new DiscordSDK(client);
    stage = "ready";
    await sdk.ready();
    stage = "authorize";
    const { code } = await sdk.commands.authorize({
      client_id: client,
      response_type: "code",
      state: "",
      prompt: "none",
      scope: ["identify"],
    });
    stage = "start";
    const r = await post("/activity/start", { code, guild_id: sdk.guildId, channel_id: sdk.channelId });
    if (r.token && r.captcha) cp.ready(r.token, r.captcha, r.y);
    else cp.fail(r.error, r.error === "auth" ? "start: token" : "");
  } catch (err) {
    cp.fail("auth", report(stage, err));
  }
})();
