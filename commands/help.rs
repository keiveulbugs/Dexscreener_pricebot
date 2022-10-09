use crate::Error;
use poise::serenity_prelude::{self as serenit, ChannelId};
use serenity::utils::Colour;

/// About command
#[poise::command(slash_command)]
pub async fn help(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    ctx.send(|b| {
        b.embed(|b| b.description(
            "This bot retrieves data from Dexscreener to show you the prices of your favourite crypto currencies.\n\n
            Use the /coin command to retrieve price data on default coins and tokens.\n\n
            Use the /address_search to search token pairs by their pairaddress.\n
            If a token is not listed as a basetoken, and only as a quotetoken, you can use *invert* to fetch price and volume of that pair address."
            ).title("help").colour(Colour::BLITZ_BLUE))
            .ephemeral(true)
            .components(|b| {
                b.create_action_row(|b| {
                    b.create_button(|b| {
                        b.label("Dexscreener")
                            .url("https://dexscreener.com/")
                            //.custom_id(1)
                            .style(serenit::ButtonStyle::Link)                        
                    })                                    
                })
            })          
    })
    .await?;
    //When the message is sent in your private channel, return the option to deregister the bot.
    if ctx.channel_id() == ChannelId(/*private channel*/) {
        poise::builtins::register_application_commands_buttons(ctx).await?;
    }
    Ok(())
}
