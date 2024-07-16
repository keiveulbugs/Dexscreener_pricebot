#![cfg(feature = "database")]
use crate::GuildCommands;
use crate::{Context, Error, DB};
use poise::serenity_prelude::{
    CreateEmbed, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use poise::CreateReply;
use serde::{Deserialize, Serialize};
use serenity::all::GuildId;

/// Check if the owner invokes this command as he can turn on slash commands across guilds
pub async fn ownercheckcommandselection(ctx: Context<'_>) -> Result<(), Error> {
    // If you are not bot owner continue like normal in your current guild
    let ownercheck = ctx.framework().options.owners.contains(&ctx.author().id);
    if !ownercheck {
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
        commandselection(ctx, guildid).await?;
        // Return the function as completed
        return Ok(());
    }

    let guilds = ctx.http().get_guilds(None, None).await?;
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
    let interaction = match message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60 * 3))
        .author_id(ctx.author().id)
        .custom_ids(vec![customid])
        .await
    {
        Some(x) => x,
        None => {
            message.delete(&ctx).await?;
            ctx.send(
                CreateReply::new()
                    .content("Command settings timed ou:/")
                    .ephemeral(true),
            )
            .await?;
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

    let guildtobechanged = GuildId::new(interactionvalue.parse::<u64>()?);
    commandselection(ctx, guildtobechanged).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableSlashcommands {
    pub availableslashcommands: Vec<String>,
    pub guildid: GuildId,
}

/// Check which commands are available in a guild to be turned on
/// If there is no database record, return watch and help
pub async fn getguildavailablecommands(guildid: GuildId) -> Result<Vec<String>, Error> {
    let optionavailableslashcommands: Option<AvailableSlashcommands> = DB
        .select(("availableslashcommands", guildid.to_string()))
        .await?;
    let availablecommands: Vec<String> = match optionavailableslashcommands {
        Some(commands) => commands.availableslashcommands,
        None => vec!["watch".to_string(), "help".to_string()],
    };
    Ok(availablecommands)
}

/// Adjust the availability of slash commands
/*
1. Get list of already active commands in guild
2. Get list of available commands in guild, if there is no record, only watch and help are available
3. Go over all commands in the bot and check if guild may use them. If so, put them as available in a selectmenu.
*/
#[allow(clippy::too_many_lines)]
async fn commandselection(ctx: Context<'_>, guildid: GuildId) -> Result<(), Error> {
    // Get the commands that are already active in the guild, these are preselected.
    let predefinedslashcommands: Vec<String> = match guildid.get_commands(ctx.http()).await {
        Ok(getcommands) => getcommands.iter().map(|com| com.name.clone()).collect(),
        Err(_) => vec![],
    };
    let customid = format!("CommandMenu{}", ctx.id());

    // Check which commands are available in a guild to be turned on
    let availablecommands = getguildavailablecommands(guildid).await?;
    // Get all the commands available in the bot.
    let commandsinframework = &ctx.framework().options().commands;

    // Loop over all available commands in framework.
    // If they are also available in this specific guild, make them a selectmenuoption
    // Settings are specifically excluded here as they should always be global, and not guild specific
    let mut selectmenuvec = vec![];
    for command in commandsinframework {
        if command.identifying_name != "settings"
            && availablecommands.contains(&command.identifying_name)
        {
            let selectmenuoption = CreateSelectMenuOption::new(
                command
                    .description
                    .clone()
                    .unwrap_or(command.identifying_name.clone()),
                command.identifying_name.clone(),
            )
            .default_selection(predefinedslashcommands.contains(&command.name));
            selectmenuvec.push(selectmenuoption);
        }
    }
    // If the number is bigger than u8, set 25 as that is the Discord limit. Also check if the u8 if bigger than 25.
    let veclen = match u8::try_from(selectmenuvec.len()) {
        Ok(number) => {
            if number >= 25 {
                25
            } else {
                number
            }
        }
        Err(_) => 25,
    };
    let message = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new()
                .content("Please select the commands that you want to be available")
                .select_menu(
                    CreateSelectMenu::new(
                        customid.clone(),
                        CreateSelectMenuKind::String {
                            options: selectmenuvec.clone(),
                        },
                    )
                    .placeholder("No Setting selected")
                    .min_values(0)
                    .max_values(veclen),
                ),
        )
        .await?;

    let interaction = match message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60 * 5))
        .author_id(ctx.author().id)
        .custom_ids(vec![customid])
        .await
    {
        Some(x) => x,
        None => {
            message.delete(&ctx).await?;
            ctx.send(
                CreateReply::new()
                    .content("Command settings timed ou:/")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    message.delete(ctx).await?;

    let interactionvalue = match &interaction.data.kind {
        poise::serenity_prelude::ComponentInteractionDataKind::StringSelect { values } => values,
        _ => panic!("unexpected interaction data kind"),
    };

    // Vec of commands that will be registered later on
    let mut guildspecificcommands: Vec<poise::serenity_prelude::CreateCommand> = vec![];
    // Vec of commands that is saved to the database for later use
    let mut dbcommands = vec![];

    'commandloop: for frameworkcommand in commandsinframework {
        let name = &frameworkcommand.identifying_name;
        if interactionvalue.contains(name) || name == "settings" {
            dbcommands.push(name.clone());

            guildspecificcommands.push(match frameworkcommand.create_as_slash_command() {
                Some(command) => command,
                None => {
                    println!("{name:#?} could not be created as slash command");
                    continue 'commandloop;
                }
            });
        }
    }
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
    // Store the commands in the database / update the stored commands in the database
    let _: Option<GuildCommands> = DB
        .update(("guildcommands", guildid.to_string()))
        .content(GuildCommands {
            guild: guildid,
            commands: dbcommands,
        })
        .await?;

    let submittedcommands = match guildid.set_commands(ctx, guildspecificcommands).await {
        Ok(val) => val,
        Err(_) => {
            ctx.send(
                    CreateReply::new()
                        .content("Something went wrong while registering commands, please try again.\nIf it still doens't work, please contact us!")
                        .ephemeral(true),
                )
                .await?;
            return Ok(());
        }
    };

    // Create a vec of embed fields that contains each registered command in current guild
    let mut commandsforembed: Vec<(String, String, bool)> = vec![];
    for command in submittedcommands {
        commandsforembed.push((command.name, command.description, false));
    }

    // Send this embed
    let embed = CreateEmbed::new()
        .fields(commandsforembed)
        .description("The following commands are available in your guild:");
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
