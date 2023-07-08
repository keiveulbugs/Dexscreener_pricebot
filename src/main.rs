mod commands;
use poise::serenity_prelude::{self as serenity};

type Error = Box<dyn std::error::Error + Send + Sync>;

#[macro_use]
//.env variables
extern crate dotenv_codegen;

//Constants
// Your Bot token
const DISCORD_TOKEN: &str = dotenv!("DISCORD_TOKEN");
// If you want to have commands specific to only a specific guild, set this as your guild_id.
const PRIVATEGUILDID: serenity::GuildId = serenity::GuildId(1014660478351454299);

async fn on_ready(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<(), Error>,
) -> Result<(), Error> {
    // To announce that the bot is online.
    println!("{} is connected!", ready.user.name);

    // This registers commands for the bot, guild commands are instantly active on specified servers
    //
    // The commands you specify here only work in your own guild!
    // This is useful if you want to control your bot from within your personal server,
    // but dont want other servers to have access to it.
    // For example sending an announcement to all servers it is located in.
    let builder = poise::builtins::create_application_commands(&framework.options().commands);
    let commands =
        serenity::GuildId::set_application_commands(&PRIVATEGUILDID, &ctx.http, |commands| {
            *commands = builder.clone();

            commands
        })
        .await;
    // This line runs on start-up to tell you which commands succesfully booted.
    println!(
        "I now have the following guild slash commands: \n{:#?}",
        commands
    );

    // Below we register Global commands, global commands can take some time to update on all servers the bot is active in
    //
    // Global commands are availabe in every server, including DM's.
    // We call the commands folder, the ping file and then the register function.
    let global_command1 =
        serenity::Command::set_global_application_commands(&ctx.http, |commands| {
            *commands = builder;
            commands
        })
        .await;
    println!(
        "I now have the following guild slash commands: \n{:#?}",
        global_command1
    );

    Ok(())
}

#[allow(unused_doc_comments)]
#[tokio::main]
async fn main() {
    // Build our client.
    let client = poise::Framework::builder()
        .token(DISCORD_TOKEN)
        .intents(serenity::GatewayIntents::empty())
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::priceinfo::address_search(),
                commands::coin::coin(),
                commands::price::price(),
                commands::help::help(),
                commands::all::all(), //commands::registration::register(),
            ],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)))
        .build()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
