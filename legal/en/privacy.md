+++
title = "Privacy policy"
description = "What the Misty website and launcher send, what we keep, for how long, and your rights."
updated = "2026-11-11"
+++

Misty has no user accounts, no ads and no analytics tools. We collect as little as possible, and almost everything stays in memory for a few minutes. This page details each case.

## Data controller

The Misty team, reachable on [Discord](https://discord.gg/rdQJUgvUh3) or on [GitHub](https://github.com/Misty-MC/Misty/issues). See the [legal notice](/legal).

## When you visit the website

- **IP address**: the site does not record it. It is only used, in memory and for one minute to one hour, to limit the number of requests and block abuse.
- **Cloudflare and the host** receive, as for any website, technical connection data (IP address, browser, requested page) to run and protect the site, under their own policies.
- **No third-party service is loaded in your browser**: fonts, scripts and images come from the site itself.

### Cookies

The site uses only two cookies, both needed for a feature you chose, and no tracking cookies:

| Cookie | Purpose | Lifetime |
| --- | --- | --- |
| `misty-lang` | Remember the chosen language | 1 year |
| `misty-player` | Remember the username entered in the shop | 1 year |

You can delete them at any time from your browser.

### Shop

The Minecraft username entered in the shop is sent to **Mojang's API** (and, if it does not answer, to **mc-heads.net**) to fetch the skin and cape. The result is kept in memory for about ten minutes. No purchase is possible for now.

## In the Misty launcher

### Updates and player count

- The launcher asks the site whether a new version exists.
- While Misty is open, it sends a sign of life every minute: an **identifier drawn at random on each start** (nothing that carries over between sessions or links to your account) and whether a game is running. It feeds the number of players online shown on the site. Nothing is written: the site forgets it after 3 minutes.

### Telemetry (only if you accept it)

On first launch, Misty asks whether you accept anonymous telemetry; you can change your mind at any time in the settings. If you accept, the launcher sends:

- an **installation identifier** drawn at random, unrelated to your account;
- the Misty version, the Windows version, the amount of RAM and the language;
- events: start, game launched, game ended or crashed, update;
- after a crash, the **last lines of the game console**, cleaned up (home folder, username and tokens removed).

Never your account, password or email address. This data is **kept for 90 days**, then deleted. A summary of the previous day's numbers and crash reports are sent to a private Discord channel of the team.

### Log reports

The console's "Send logs" button sends, at your request, the last lines of the console cleaned up the same way, even without telemetry. You get a code to give on Discord so the team can find the report. Reports are **kept for 90 days** and forwarded to the team's private Discord channel.

### Misty Client (in game)

Misty Client, the launcher's in-game mod, lets Misty players recognize each other (Misty badge, capes, elytra and hats). It is on by default and can be turned off in the launcher settings. When it is on:

- your account is verified the way a Minecraft server does it: your **username** is checked with **Mojang**, which returns your Minecraft **UUID** to us. Your Minecraft token never leaves your game;
- the game sends a sign of life every minute with the **Misty cosmetics you wear**;
- for the "early player" cape, kept for the first players, we keep your **UUID** and the **date you first signed in** to Misty Client. Nothing else (no username, no IP address);
- it asks which players around you use Misty (their UUIDs, 100 at most). These lists are not kept.

The rest stays in memory: a player is forgotten 3 minutes after their last sign of life, a login after one hour of inactivity. Offline accounts cannot be verified and are not shown to others.

### Microsoft account

Signing in to your Microsoft account happens directly between the launcher and Microsoft. The site receives nothing from it.

## Discord verification (Misty Bot)

On Discord servers that use Misty Bot, new members are verified through the site:

- the verification link contains the **Discord IDs** of the server, the member and the role to give;
- from inside Discord (Activity), the site asks Discord who you are (`identify` permission: Discord ID and username);
- once the captcha is solved, the site gives you the role and may log the verification in the log channel chosen by the server.

The captcha and its state stay in memory for 10 minutes at most. For each server, the site only keeps its configuration (role to give, log channel, language).

## Legal bases

- **Legitimate interest**: running the website, the launcher and Misty Client, and protecting them against abuse.
- **Consent**: telemetry, which you can withdraw at any time in the settings.
- **Your request**: the log reports you send and Discord verification.

## How long

| Data | Lifetime |
| --- | --- |
| IP address (rate limiting) | 1 minute to 1 hour, in memory |
| Launcher sign of life | 3 minutes, in memory |
| Misty Client | 3 minutes to 1 hour, in memory |
| Misty Client: UUID and first sign-in date (early player cape) | as long as the early player cape exists |
| Shop profiles | about 10 minutes, in memory |
| Verification captcha | 10 minutes, in memory |
| Telemetry and log reports | 90 days |
| Discord server configuration | as long as the server uses Misty Bot |
| Cookies | 1 year |

## Sharing

We do not sell or rent any data. It only goes through the services needed to run Misty: the host (netcup, in Germany), Cloudflare, Mojang (account verification and skins), mc-heads.net (skins, as a fallback), GitHub (launcher files) and Discord (verification, the team's private channel). Some of these services are located in the United States.

## Your rights

You can ask to access, correct or delete your data, and object to its processing, by writing to us on [Discord](https://discord.gg/rdQJUgvUh3). Since telemetry and reports are not linked to any account, give us the installation identifier or the report code so we can find them.

You can also lodge a complaint with the [CNIL](https://www.cnil.fr/en/complaints) or with the data protection authority of your country.

## Changes

This page may change along with Misty. The date of the last update is shown at the top of the page.
