#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(
    clippy::single_match_else,
    clippy::manual_let_else,
    clippy::module_inception
)]
use macro_env::envseeker;
use serde::{Deserialize, Serialize};

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CacheHttp;

use serenity::builder::CreateCommand;

mod commands;
#[cfg(feature = "database")]
mod settings;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
#[derive(Debug, Clone)]
pub struct Data {}

#[cfg(feature = "database")]
use ::{
    once_cell::sync::Lazy,
    surrealdb::{
        engine::local::{Db, File},
        Surreal,
    },
};
#[cfg(feature = "database")]
static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

#[cfg(feature = "database")]
#[derive(Debug, Serialize, Deserialize)]
pub struct GuildCommands {
    pub guild: poise::serenity_prelude::GuildId,
    pub commands: Vec<String>,
}

/// Register the specific commands for every guild.
/// The default is registering all commands.
/// Server admins can turn off which commands should be visible.
/// Bot owners can turn off which commands are available in a server.
async fn on_ready(
    ctx: &serenity::prelude::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<Data, Error>,
) -> Result<(), Error> {
    // Get all available commands in framework
    let commandsinframework = &framework.options().commands;
    let commandnamesinframework: Vec<String> = commandsinframework
        .iter()
        .map(|x| x.identifying_name.clone())
        .collect();

    let guilds = ctx.http().get_guilds(None, None).await?;
    println!("The bot is in these guilds:\n{guilds:#?}");

    // Go over all guilds to register the commands in all of them
    for guild in guilds {
        // If database is turned on, check database if the guild has specific commands available, otherwise make all commands available.
        if cfg!(feature = "database") {
            // We get the saved commands for each guild by fetching them from the database, and push them into a vec of GuildCommands
            #[cfg(feature = "database")]
            {
                let guildspecificcommands: Option<GuildCommands> =
                    DB.select(("guildcommands", guild.id.to_string())).await?;
                let commandstoturnon = match guildspecificcommands {
                    Some(com) => com.commands,
                    None => commandnamesinframework.clone(),
                };
                let commandregistery = specificcommandfinder(commandstoturnon, commandsinframework);
                // Register commands for this guild
                let commandsubmitting = guild.id.set_commands(ctx, commandregistery).await;
                println!("Submitting commands:\n{commandsubmitting:#?}");
            }
        } else {
            let commandregistery =
                specificcommandfinder(commandnamesinframework.clone(), commandsinframework);
            // Register commands for this guild
            let commandsubmitting = guild.id.set_commands(ctx, commandregistery).await;
            println!("Submitting commands:\n{commandsubmitting:#?}");
        };
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)] //Clippy doesn't like the Vec<String> for commands in database, but this is needed with SurrealDB
/// Match saved value with commands available, and return this in a vector of commands.
fn specificcommandfinder(
    commandsindatabase: Vec<String>,
    commandsinframework: &Vec<poise::Command<Data, Box<dyn std::error::Error + Send + Sync>>>,
) -> Vec<CreateCommand> {
    let mut guildspecificcommands: Vec<CreateCommand> = vec![];
    'commandloop: for frameworkcommand in commandsinframework {
        let name = &frameworkcommand.identifying_name;
        if commandsindatabase.contains(name) {
            guildspecificcommands.push(match frameworkcommand.create_as_slash_command() {
                Some(command) => command,
                None => {
                    println!("{name} could not be created as slash command");
                    continue 'commandloop;
                }
            });
        }
    }
    guildspecificcommands
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "database")]
    createdatabase().await;

    println!("Starting bot");
    // Set GUILDS to be an intent as we require it for having custom commands
    let intents = serenity::GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                #[cfg(feature = "database")]
                settings::settings::settings(),
                #[cfg(feature = "database")]
                commands::price::price(),
                #[cfg(not(feature = "database"))]
                commands::pricewithoutdb::price(),
            ],
            ..Default::default()
        })
        .setup(
            move |ctx: &::serenity::prelude::Context, ready, framework| {
                Box::pin(async move {
                    println!("Logged in as {}", ready.user.name);

                    let _ = on_ready(ctx, ready, framework).await;
                    println!("The bot is done getting ready");

                    Ok(Data {})
                })
            },
        )
        .build();

    let mut client = match serenity::ClientBuilder::new(
        // envseeker will try to find the variable in .env, then in system variables and finally will ask you for it if it doesn't find any.
        envseeker(macro_env::SearchType::All, "DEXSCREENER_BOT"),
        intents,
    )
    .framework(framework)
    .await
    {
        Ok(client) => client,
        Err(clientcreationerror) => {
            println!("Couldn't create the Discord bot client:\n{clientcreationerror}");
            return;
        }
    };

    // Start client, show error if it fails.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

#[cfg(feature = "database")]
async fn createdatabase() {
    println!("Creating database");

    match DB.connect::<File>("dexscreener.db").await {
        Ok(val) => val,
        Err(dbconnecterror) => panic!("failed to connect to database: {dbconnecterror}"),
    };
    match DB.use_ns("dexscreener").use_db("dexscreenerdb").await {
        Ok(val) => val,
        Err(dberror) => panic!("failed to use namespace or datebase: {dberror}"),
    };
}
