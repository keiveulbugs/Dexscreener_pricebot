#![cfg(feature = "database")]
#![allow(clippy::module_name_repetitions)]
use crate::{Context, Error, DB};
use poise::serenity_prelude::{
    CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use poise::CreateReply;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableSettings {
    // Ability to change which commands are available in a guild
    availablecommands: bool,
    // Ability to change which commands are available in `availablecommands` (should be only owners of the bot)
    owneravailablecommands: bool,
    // Ability to change which tokens are being tracked in price tracking
    tokenpricetracking: bool,
}

/// Change settings depending on your server
//
// 1. Check for Admins or Owners
// 2. Get the GuildId, which is used to get the settings set for a guild
// 3. Retrieve the available commands for a GuildId and put them in a vec, or fall back to default
// 4. Open a context menu with the available settings menu
// 5. Delete context menu and start that setting
#[allow(clippy::too_many_lines)]
#[poise::command(slash_command)]
pub async fn settings(ctx: Context<'_>) -> Result<(), Error> {
    let ownercheck = ctx.framework().options.owners.contains(&ctx.author().id);
    let admincheck = ctx
        .author_member()
        .await
        .and_then(|m| m.permissions)
        .is_some_and(poise::serenity_prelude::Permissions::administrator);

    // Check if the author is the bot owner or an admin of that server.
    if !admincheck && !ownercheck {
        ctx.send(
            CreateReply::default()
                .content(
                    "Sorry, you are not allowed to use this command.
        \nOnly administrators and bot owners are.",
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    };
    ctx.send(
        CreateReply::default()
            .content("Opening settings menu")
            .ephemeral(true),
    )
    .await?;

    let guildid = match ctx.guild_id() {
        Some(guildid) => guildid,
        None => {
            ctx.send(
                CreateReply::new()
                    .content("It looks you are not in a server")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let dbcommandpermissions: Option<AvailableSettings> = DB
        .select(("availablesettings", guildid.to_string()))
        .await?;
    let mut commandpermissions: AvailableSettings = match dbcommandpermissions {
        Some(permissions) => permissions,
        None => AvailableSettings {
            availablecommands: true,
            owneravailablecommands: false,
            tokenpricetracking: false,
        },
    };
    if ownercheck {
        commandpermissions = AvailableSettings {
            availablecommands: true,
            owneravailablecommands: true,
            tokenpricetracking: true,
        }
    };

    let mut selectmenuvec = vec![];

    if commandpermissions.availablecommands {
        selectmenuvec.push(CreateSelectMenuOption::new(
            "Activate or deactivate available slash commands",
            "availablecommands",
        ));
    }
    if commandpermissions.owneravailablecommands {
        selectmenuvec.push(CreateSelectMenuOption::new(
            "(De)Activate commands available to turn on for guilds",
            "owneravailablecommands",
        ));
    }
    if commandpermissions.tokenpricetracking {
        selectmenuvec.push(CreateSelectMenuOption::new(
            "Add a token of which the price can be fetched",
            "tokenpricetracking",
        ));
    }

    let customid = format!("SettingsMenu{}", ctx.id());
    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new()
                .content("Please click the setting you want to change")
                .select_menu(
                    CreateSelectMenu::new(
                        "settings_menu",
                        CreateSelectMenuKind::String {
                            options: selectmenuvec,
                        },
                    )
                    .custom_id(customid.clone())
                    .max_values(1)
                    .placeholder("No Setting chosen"),
                ),
        )
        .await?;
    let interaction = match message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60 * 3))
        .author_id(ctx.author().id)
        .custom_ids(vec![customid])
        .await
    {
        Some(x) => x,
        None => {
            message.reply(&ctx, "Timed out").await?;
            return Ok(());
        }
    };

    message.delete(ctx).await?;

    let interactionvalue = match &interaction.data.kind {
        poise::serenity_prelude::ComponentInteractionDataKind::StringSelect { values } => {
            &values[0]
        }
        _ => panic!("unexpected interaction data kind"),
    };

    match interactionvalue.as_str() {
        "availablecommands" => {
            crate::settings::commandselection::ownercheckcommandselection(ctx).await?;
        }
        "owneravailablecommands" => {
            crate::settings::owneravailablecommands::ownercheckavailablecommands(ctx).await?;
        }
        "tokenpricetracking" => {}
        _ => {
            return Err(format!(
                "There is no implementation available for this setting: {interactionvalue}."
            )
            .into())
        }
    }
    Ok(())
}
