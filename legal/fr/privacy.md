+++
title = "Politique de confidentialité"
description = "Ce que le site et le launcher Misty envoient, ce que nous gardons, combien de temps, et vos droits."
updated = "2026-11-10"
+++

Misty n'a ni compte utilisateur, ni publicité, ni outil de mesure d'audience. Nous collectons le moins possible, et presque tout reste en mémoire quelques minutes. Cette page détaille chaque cas.

## Responsable du traitement

L'équipe Misty, joignable sur [Discord](https://discord.gg/rdQJUgvUh3) ou sur [GitHub](https://github.com/Misty-MC/Misty/issues). Voir les [mentions légales](/legal).

## En visitant le site

- **Adresse IP** : le site ne l'enregistre pas. Elle sert seulement, en mémoire et pendant une minute à une heure, à limiter le nombre de requêtes et à bloquer les abus.
- **Cloudflare et l'hébergeur** reçoivent, comme pour tout site web, des données techniques de connexion (adresse IP, navigateur, page demandée) pour faire fonctionner et protéger le site, selon leurs propres politiques.
- **Aucun service tiers n'est chargé dans votre navigateur** : polices, scripts et images viennent du site lui-même.

### Cookies

Le site n'utilise que deux cookies, tous deux nécessaires à une fonction que vous avez choisie, et aucun cookie de suivi :

| Cookie | Rôle | Durée |
| --- | --- | --- |
| `misty-lang` | Retenir la langue choisie | 1 an |
| `misty-player` | Retenir le pseudo saisi dans la boutique | 1 an |

Vous pouvez les supprimer à tout moment depuis votre navigateur.

### Boutique

Le pseudo Minecraft saisi dans la boutique est envoyé à l'**API de Mojang** (et, si elle ne répond pas, à **mc-heads.net**) pour récupérer le skin et la cape. Le résultat est gardé en mémoire une dizaine de minutes. Aucun achat n'est possible pour l'instant.

## Dans le launcher Misty

### Mises à jour et compteur de joueurs

- Le launcher demande au site si une nouvelle version existe.
- Tant que Misty est ouvert, il envoie toutes les minutes un signe de vie : un **identifiant tiré au hasard à chaque démarrage** (rien qui reste d'une session à l'autre ni qui relie à votre compte) et le fait qu'une partie soit en cours ou non. Cela sert au nombre de joueurs en ligne affiché sur le site. Rien n'est écrit : le site l'oublie après 3 minutes.

### Télémétrie (seulement si vous l'acceptez)

Au premier lancement, Misty vous demande si vous acceptez la télémétrie anonyme ; vous pouvez changer d'avis à tout moment dans les options. Si vous l'acceptez, le launcher envoie :

- un **identifiant d'installation** tiré au hasard, sans lien avec votre compte ;
- la version de Misty, la version de Windows, la quantité de RAM et la langue ;
- les événements : démarrage, partie lancée, partie terminée ou plantée, mise à jour ;
- en cas de plantage, les **dernières lignes de la console** du jeu, nettoyées (dossier personnel, pseudo et jetons retirés).

Jamais votre compte, votre mot de passe ni votre adresse e-mail. Ces données sont **conservées 90 jours**, puis supprimées. Un résumé chiffré de la veille et les rapports de plantage sont envoyés dans un salon Discord privé de l'équipe.

### Rapports de logs

Le bouton « Envoyer les logs » de la console envoie, à votre demande, les dernières lignes de la console nettoyées de la même manière, même sans télémétrie. Vous recevez un code à donner sur Discord pour que l'équipe retrouve le rapport. Les rapports sont **conservés 90 jours** et transmis au salon Discord privé de l'équipe.

### Misty Client (en jeu)

Misty Client, le mod en jeu du launcher, permet aux joueurs Misty de se reconnaître (badge, cape et élytres Misty). Il est activé par défaut et se désactive dans les options du launcher. Quand il est actif :

- votre compte est vérifié comme le ferait un serveur Minecraft : votre **pseudo** est vérifié auprès de **Mojang**, qui nous renvoie votre **UUID** Minecraft. Votre jeton Minecraft ne quitte jamais votre jeu ;
- le jeu envoie toutes les minutes un signe de vie avec le **cosmétique Misty porté** ;
- il demande quels joueurs autour de vous utilisent Misty (leurs UUID, 100 au plus). Ces listes ne sont pas gardées.

Tout reste en mémoire : un joueur est oublié 3 minutes après son dernier signe de vie, une connexion après une heure d'inactivité. Les comptes hors ligne ne peuvent pas être vérifiés et ne sont pas visibles des autres.

### Compte Microsoft

La connexion à votre compte Microsoft se fait directement entre le launcher et Microsoft. Le site n'en reçoit rien.

## Vérification Discord (Misty Bot)

Sur les serveurs Discord qui utilisent Misty Bot, la vérification des nouveaux membres passe par le site :

- le lien de vérification contient les **identifiants Discord** du serveur, du membre et du rôle à donner ;
- depuis Discord (Activité), le site demande à Discord qui vous êtes (autorisation `identify` : identifiant et pseudo Discord) ;
- une fois le captcha réussi, le site vous donne le rôle et peut noter la vérification dans le salon de logs choisi par le serveur.

Le captcha et son état restent en mémoire 10 minutes au plus. Le site garde seulement, pour chaque serveur, sa configuration (rôle à donner, salon de logs, langue).

## Bases légales

- **Intérêt légitime** : faire fonctionner le site, le launcher et Misty Client, et les protéger contre les abus.
- **Consentement** : la télémétrie, que vous pouvez retirer à tout moment dans les options.
- **Votre demande** : les rapports de logs que vous envoyez et la vérification Discord.

## Combien de temps

| Donnée | Durée |
| --- | --- |
| Adresse IP (limitation des requêtes) | 1 minute à 1 heure, en mémoire |
| Signe de vie du launcher | 3 minutes, en mémoire |
| Misty Client | 3 minutes à 1 heure, en mémoire |
| Profils de la boutique | 10 minutes environ, en mémoire |
| Captcha de vérification | 10 minutes, en mémoire |
| Télémétrie et rapports de logs | 90 jours |
| Configuration des serveurs Discord | tant que le serveur utilise Misty Bot |
| Cookies | 1 an |

## Partage des données

Nous ne vendons ni ne louons aucune donnée. Elles ne passent que par les services nécessaires au fonctionnement : l'hébergeur, Cloudflare, Mojang (vérification des comptes et skins), mc-heads.net (skins, en secours), GitHub (fichiers du launcher) et Discord (vérification, salon privé de l'équipe). Certains de ces services sont situés aux États-Unis.

## Vos droits

Vous pouvez demander l'accès à vos données, leur rectification ou leur suppression, et vous opposer à leur traitement, en nous écrivant sur [Discord](https://discord.gg/rdQJUgvUh3). Comme la télémétrie et les rapports ne sont reliés à aucun compte, donnez-nous l'identifiant d'installation ou le code du rapport pour que nous les retrouvions.

Vous pouvez aussi adresser une réclamation à la [CNIL](https://www.cnil.fr/fr/plaintes) ou à l'autorité de protection des données de votre pays.

## Modifications

Cette page peut évoluer avec Misty. La date de dernière mise à jour figure en haut de la page.
