+++
title = "Accounts"
description = "Sign in with one or several Microsoft accounts, or play with offline accounts."
section = "play"
+++

Misty supports two kinds of accounts. You pick the one to play with in the **Account** list of the home page.

## Microsoft accounts

A Microsoft account that owns **Minecraft: Java Edition** is needed to join online servers, and to change your [skin and capes](/docs/skins).

1. On the home page, open the **Account** list and choose **Sign in with a Microsoft account** (or use the Microsoft account card in [Settings](/docs/settings)).
2. A Microsoft sign-in window opens. Sign in as you would on the web.
3. Once signed in, the account shows in the list with your skin's head.

You can add **several Microsoft accounts** and switch between them from the home page or from the settings. **Forget this account** removes one account without touching the others.

> [!NOTE]
> **Full sign-out** also forgets the account cached on this PC. Use it on a shared computer.

### When sign-in fails

| Message | What to do |
|---|---|
| *This Microsoft account has no Xbox profile* | Sign in once on minecraft.net to create the profile. |
| *Xbox Live is not available in this account's country* | The account's country does not allow Xbox Live. |
| *This account must first be age-verified on xbox.com* | Complete the age verification on xbox.com. |
| *This is a child account* | An adult must add the account to a Microsoft family. |
| *This account does not own Minecraft Java Edition* | Buy the game, or sign in with the account that owns it. |

## Offline accounts

Offline accounts are usernames you can play with without a Microsoft account. Manage them in the **Offline accounts** card of [Settings](/docs/settings): type a username and click **Add**.

At the first launch, Misty creates an offline account called *Misty*.

> [!WARNING]
> Servers that check accounts (most public servers) refuse offline accounts. Offline accounts are meant for single player and for servers that accept them.

## Where accounts are stored

- Microsoft accounts: `msa.json`
- Offline accounts: `listpseudo.txt`

Both are in `%LOCALAPPDATA%\Misty`. See [Files and folders](/docs/files-and-folders).
