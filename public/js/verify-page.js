// /verify dans le navigateur : le lien et le premier puzzle sont déjà dans la page.
import { captcha } from "/js/captcha.js";

const data = document.getElementById("cp-data").dataset;
captcha().ready(data.token, data.captcha, +data.y);
