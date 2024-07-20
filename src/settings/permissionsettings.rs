#![cfg(feature = "database")]
use crate::settings::dbstructs::AvailableSettings;
use crate::{Context, Error, DB};
use poise::serenity_prelude::CreateMessage;
use poise::serenity_prelude::CreateSelectMenu;
use poise::serenity_prelude::CreateSelectMenuKind;
use poise::serenity_prelude::CreateSelectMenuOption;
use poise::CreateReply;

use super::commonfunctions::getguildid;

/// Setting the permissions a guild has
pub async fn permissionsettings(ctx: Context<'_>) -> Result<(), Error> {
    let guildid = getguildid(ctx).await?;

    let dbcommandpermissions: Option<AvailableSettings> = DB
        .select(("availablesettings", guildid.to_string()))
        .await?;

    let commandpermissions: AvailableSettings = match dbcommandpermissions {
        Some(permissions) => permissions,
        None => AvailableSettings {
            availablecommands: true,
            owneravailablecommands: false,
            tokenpricetracking: true,
            globaltokens: false,
        },
    };
    let selectmenuvec = vec![
        CreateSelectMenuOption::new(
            "Change which tokens are registered in a guild",
            "availablecommands",
        )
        .default_selection(commandpermissions.availablecommands),
        CreateSelectMenuOption::new("Add tokens to be tracked in /price", "tokenpricetracking")
            .default_selection(commandpermissions.tokenpricetracking),
        CreateSelectMenuOption::new("Be able to set tokens to be global", "globaltokens")
            .default_selection(commandpermissions.globaltokens),
    ];

    let customid = format!("permissionsettings{}", ctx.id());
    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new()
                .content("Please click the permissions you want to give this guild")
                .select_menu(
                    CreateSelectMenu::new(
                        &customid,
                        CreateSelectMenuKind::String {
                            options: selectmenuvec,
                        },
                    )
                    .max_values(3)
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

    let interactionvalue = match &interaction.data.kind {
        poise::serenity_prelude::ComponentInteractionDataKind::StringSelect { values } => {
            values.clone()
        }
        _ => panic!("unexpected interaction data kind"),
    };

    message.delete(ctx).await?;

    let selectedpermissions = AvailableSettings {
        availablecommands: interactionvalue.contains(&"availablecommands".to_string()),
        owneravailablecommands: false,
        tokenpricetracking: interactionvalue.contains(&"tokenpricetracking".to_string()),
        globaltokens: interactionvalue.contains(&"globaltokens".to_string()),
    };

    let dbresult: Option<AvailableSettings> = DB
        .update(("availablesettings", guildid.to_string()))
        .content(selectedpermissions)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Set for {} the following permissions:\n{:#?}",
                guildid.name(ctx.cache()).unwrap_or(guildid.to_string()),
                dbresult
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
