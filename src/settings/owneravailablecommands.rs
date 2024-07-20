#![cfg(feature = "database")]
use crate::{Context, Error, DB};
use poise::serenity_prelude::{
    CreateEmbed, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use poise::CreateReply;
use serde::{Deserialize, Serialize};
use serenity::all::GuildId;

use super::commonfunctions::ownercheck;

/// Check if the owner invokes this command as he can make slash commands available for guilds
pub async fn ownercheckavailablecommands(ctx: Context<'_>) -> Result<(), Error> {
    if !ownercheck(ctx, Some("You are not the bot owner!")).await? {
        // This is probably not needed as ownercheck Errors when you are not the owner. And thus returning early from the function.
        return Ok(());
    }
    let guildtobechanged = guildselectmenu(ctx).await?;

    availablecommandselection(ctx, guildtobechanged).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableSlashcommands {
    availableslashcommands: Vec<String>,
    guildid: GuildId,
}

/// Set commands to be available in a guild to turn on or off. Only available by owners.
async fn availablecommandselection(ctx: Context<'_>, guildid: GuildId) -> Result<(), Error> {
    // Check which commands are available in a guild to be turned on
    let availablecommands =
        crate::settings::commandselection::getguildavailablecommands(guildid).await?;

    // Get all the commands available in the bot.
    let commandsinframework = &ctx.framework().options().commands;

    let mut selectmenuvec = vec![];
    for command in commandsinframework {
        let name = &command.identifying_name;
        selectmenuvec.push(
            CreateSelectMenuOption::new(name, name)
                .default_selection(availablecommands.contains(name)),
        );
    }
    let customid = format!("availablecommandsmenu{}", ctx.id());

    let selectmenuveclength = if selectmenuvec.len() <= 25 {
        u8::try_from(selectmenuvec.len()).unwrap_or(25)
    } else {
        ctx.send(
            CreateReply::new()
                .content("Only showing the first 25 commands")
                .ephemeral(true),
        )
        .await?;
        25
    };

    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new()
                .content("Select the commands that should be available")
                .select_menu(
                    CreateSelectMenu::new(
                        customid.clone(),
                        CreateSelectMenuKind::String {
                            options: selectmenuvec.clone(),
                        },
                    )
                    .min_values(0)
                    .max_values(selectmenuveclength)
                    .placeholder("No commands chosen"),
                ),
        )
        .await?;
    let Some(interaction) = message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60 * 3))
        .author_id(ctx.author().id)
        .custom_ids(vec![customid])
        .await
    else {
        message.delete(&ctx).await?;
        ctx.send(
            CreateReply::new()
                .content("Command settings timed ou:/")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    message.delete(ctx).await?;

    let poise::serenity_prelude::ComponentInteractionDataKind::StringSelect {
        values: interactionvalue,
    } = &interaction.data.kind
    else {
        panic!("unexpected interaction data kind")
    };

    let mut commandstobesetavailable = vec![];
    for commands in commandsinframework {
        if interactionvalue.contains(&commands.identifying_name) {
            commandstobesetavailable.push(commands.identifying_name.clone());
        }
    }

    let optionavailableslashcommands: Option<AvailableSlashcommands> = DB
        .update(("availableslashcommands", guildid.to_string()))
        .content({
            AvailableSlashcommands {
                availableslashcommands: commandstobesetavailable,
                guildid,
            }
        })
        .await?;

    // Create a vec of embed fields that contains each registered command in current guild
    let mut commandsforembed: Vec<(String, String, bool)> = vec![];

    if let Some(dbavailableslashcommands) = optionavailableslashcommands {
        for command in dbavailableslashcommands.availableslashcommands {
            commandsforembed.push((command, String::new(), false));
        }
    }

    // Send this embed
    let embed = CreateEmbed::new()
        .fields(commandsforembed)
        .description(format!(
            "The following commands are available in {:#?}:",
            guildid.name(ctx).unwrap_or("Unknown".to_string())
        ));
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Create a select menu including all guilds.
/// The user can choose one guild of which the guildid is returned.
pub async fn guildselectmenu(ctx: Context<'_>) -> Result<GuildId, Error> {
    let guilds = match ctx.http().get_guilds(None, None).await {
        Ok(guildsfetch) => guildsfetch,
        Err(errorguildsfetch) => {
            return Err(
                format!("The bot was not able to get the guilds: {errorguildsfetch}").into(),
            )
        }
    };
    let mut vecofguildmenu = vec![];
    for guild in guilds {
        vecofguildmenu.push(CreateSelectMenuOption::new(
            guild.name,
            guild.id.to_string(),
        ));
    }

    let customid = format!("guildmenu{}", ctx.id());
    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new()
                .content("Please click the guild you want to change")
                .select_menu(
                    CreateSelectMenu::new(
                        customid.clone(),
                        CreateSelectMenuKind::String {
                            options: vecofguildmenu,
                        },
                    )
                    .max_values(1)
                    .placeholder("No guild chosen"),
                ),
        )
        .await?;
    let Some(interaction) = message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60 * 3))
        .author_id(ctx.author().id)
        .custom_ids(vec![customid])
        .await
    else {
        message.delete(&ctx).await?;
        ctx.send(
            CreateReply::new()
                .content("Command settings timed out:/")
                .ephemeral(true),
        )
        .await?;
        return Err("Command settings timed out:/".into());
    };

    message.delete(ctx).await?;

    let interactionvalue = match &interaction.data.kind {
        poise::serenity_prelude::ComponentInteractionDataKind::StringSelect { values } => {
            &values[0]
        }
        _ => panic!("unexpected interaction data kind"),
    };

    let guildtobechanged = GuildId::new(interactionvalue.parse::<u64>()?);
    Ok(guildtobechanged)
}
