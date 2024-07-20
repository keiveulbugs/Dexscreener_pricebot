use poise::CreateReply;

use crate::{Context, Error};
/// Checks if a user is part of the botowner's team, returning a bool.
/// If text is supplied, this send as an ephemeral message to the user with the supplied text.
pub async fn ownercheck(ctx: Context<'_>, text: Option<String>) -> Result<bool, Error> {
    let ownercheck = ctx.framework().options.owners.contains(&ctx.author().id);
    if !ownercheck && text.is_some() {
        match ctx.send(
            CreateReply::new()
                .content(text.unwrap_or("You are not the bot owner. You don't have the right permissions for this action.".to_string()))
                .ephemeral(true),
        )
        .await {
            Ok(botownercheckmessage) => botownercheckmessage,
            Err(botownercheckerror) => return Err(format!("Person is not the bot owner, however, this message could not be send to them.\nError: {botownercheckerror}").into())
        };
    }
    Ok(ownercheck)
}
