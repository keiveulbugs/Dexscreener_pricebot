use poise::CreateReply;
use serenity::all::GuildId;

use crate::{Context, Error};
/// Checks if a user is part of the botowner's team, returning a bool.
/// If text is supplied, this send as an ephemeral message to the user with the supplied text.
pub async fn ownercheck(ctx: Context<'_>, text: Option<&str>) -> Result<bool, Error> {
    let ownercheck = ctx.framework().options.owners.contains(&ctx.author().id);
    if !ownercheck && text.is_some() {
        match ctx.send(
            CreateReply::new()
                .content(text.unwrap_or("You are not the bot owner. You don't have the right permissions for this action."))
                .ephemeral(true),
        )
        .await {
            Ok(botownercheckmessage) => botownercheckmessage,
            Err(botownercheckerror) => return Err(format!("Person is not the bot owner, however, this message could not be send to them.\nError: {botownercheckerror}").into())
        };
    }
    Ok(ownercheck)
}
/// Checks if a user is admin of this guild, returning a bool.
/// If text is supplied, this send as an ephemeral message to the user with the supplied text.
pub async fn admincheck(ctx: Context<'_>, text: Option<&str>) -> Result<bool, Error> {
    let admincheck = ctx
        .author_member()
        .await
        .and_then(|m| m.permissions)
        .is_some_and(poise::serenity_prelude::Permissions::administrator);

    if !admincheck {
        match ctx.send(
            CreateReply::new()
                .content(text.unwrap_or("You are not the bot owner. You don't have the right permissions for this action."))
                .ephemeral(true),
        )
        .await {
            Ok(botownercheckmessage) => botownercheckmessage,
            Err(botownercheckerror) => return Err(format!("Person is not an admin on this guild, however, this message could not be send to them.\nError: {botownercheckerror}").into())
        };
    }
    Ok(admincheck)
}

/// Get the guildid. Return an error if it can not be found and send an ephemeral message to the suer about it.
pub async fn getguildid(ctx: Context<'_>) -> Result<GuildId, Error> {
    match ctx.guild_id() {
        Some(guildid) => Ok(guildid),
        None => {
            ctx.send(
                CreateReply::new()
                    .content("It looks you are not in a server")
                    .ephemeral(true),
            )
            .await?;
            Err("User is not in a server or there were problems fetching the GuildId".into())
        }
    }
}
