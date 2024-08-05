# Dexscreener.com Pricebot
*After a long wait, the bot is now updated to the latest Poise and Serenity versions. Alongside this various improvements have been made. TLDR: The whole bot is rewritten from the ground up.*

## Features
**Default:** This is a simple pricebot where you can get the price of the hardcoded coins in `pricewithoutdb.rs`. You can change the hashmap to include more or different coins/tokens. When users enter a symbol that is not in there, they will get the option to fetch it manually with a modal.

**Database:** This allows a lot more customization. It has the following features:
- Store coins/tokens in a database to show as autocomplete suggestions
- Turn on and off commands visible in a guild/server
- Make commands available or not available in a guild/server (As of right now, it does not set global commands so it only registers commands in joined guilds on start up. You can manually turn on commands while running through the command.)
- Give certain permissions extra rights

## Running
For the basic lightweight version run:

```cargo run --release```

For the database version run:

```cargo run --release --features database```

Both cases, run can be changed to build to create a binary.

The bot will ask on startup the bot secret, but you can also set it using a `.env` with `DEXSCREENER_BOT` as variable name.


## Good to know
- The bot will take the pair with the most volume based on the smart contract address of a token
- Clippy is set to pedantic to avoid bad code. But it whines about everything, so have allowed quite a bit (i.e. struct names being too similar to filenames)
- All rights are reserved to Dexscreener.com


# ToDo
- [ ] Automatically register commands when the bot joins a new guild.
- [ ] Make an owner command that allows setting tokens from a guild global or private