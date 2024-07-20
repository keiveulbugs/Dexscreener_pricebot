#![cfg(feature = "database")]
use poise::serenity_prelude::GuildId;
use serde::{Deserialize, Serialize};

/// Which settings someone can change in the bot.
/// - `availablecommands`: Turn on and off which commands are visible for users in a guild
/// - `owneravailablecommands`: Change which commands are available to turn on for `availablecommands`
/// - `tokenpricetracking`: Allow adding tokens to `/price`-autocomplete through settings
/// - `globaltoken`: Allow tokens added through `tokenpricetracking` to be available in all guilds
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableSettings {
    // Ability to change which commands are available in a guild
    pub availablecommands: bool,
    // Ability to change which commands are available in `availablecommands` (should be only owners of the bot)
    pub owneravailablecommands: bool,
    // Ability to change which tokens are being tracked in price tracking
    pub tokenpricetracking: bool,
    // Ability to set tokens globally
    pub globaltokens: bool,
}

/// The commands that are available in a guild to be turned on.
/// These are registered and non-registered commands.
#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableSlashcommands {
    pub availableslashcommands: Vec<String>,
    pub guildid: GuildId,
}

/// The commands that are registered in a guild (and thus visibile)
#[derive(Debug, Serialize, Deserialize)]
pub struct GuildCommands {
    pub guildid: GuildId,
    pub commands: Vec<String>,
}
