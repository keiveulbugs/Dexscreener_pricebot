use poise::CreateReply;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbed};

use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn price(ctx: Context<'_>) -> Result<(), Error> {
    let text = if cfg!(feature = "database") {
        "You can fetch the latest prices from dexscreener using `/price`\n
        Some tokens are autocompleted. If this is not available, the bot will offer the option to search by smart contract address.\n
        Your admin is capable to set more tokens to be autocompleted for your guild."
    } else {
        "You can fetch the latest prices from dexscreener using `/price`\n
        Some tokens are autocompleted. If this is not available, the bot will offer the option to search by smart contract address."
    };

    let embed = CreateEmbed::default().description(text).title("Help menu");
    let button = CreateButton::new_link("https://github.com/keiveulbugs/Dexscreener_pricebot")
        .label("The Git repository");
    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![CreateActionRow::Buttons(vec![button])]),
    )
    .await?;

    Ok(())
}
