#![cfg(feature = "database")]
use crate::commands::price::Coins;
use crate::{Context, Data, Error, DB};
use poise::{CreateReply, Modal};
use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use serenity::{
    all::{CreateActionRow, CreateButton, CreateEmbed},
    model::application::ComponentInteraction,
};

#[derive(Debug, Modal, Clone)]
#[name = "Add a custom token"]
struct AddToken {
    #[name = "Enter address of the token"] // Field name by default
    #[placeholder = "0x....."] // No placeholder by default
    #[min_length = 42] // No length restriction by default (so, 1-4000 chars)
    #[max_length = 42]
    address: String,
    #[name = "Add a url that links to the logo"]
    #[placeholder = "https://example.com/logo.png"]
    logo: Option<String>,
}

#[allow(clippy::too_many_lines)]
pub async fn addtoken(
    ctx: Context<'_>,
    interaction: ComponentInteraction,
    guildid: GuildId,
    globaltokenpermission: bool,
) -> Result<(), Error> {
    let modalresponse = match poise::execute_modal_on_component_interaction::<AddToken>(
        ctx.serenity_context(),
        interaction,
        None,
        None,
    )
    .await?
    {
        Some(val) => val,
        None => {
            ctx.say("No datat found in modal").await?;
            return Ok(());
        }
    };

    let url = format!(
        "https://api.dexscreener.com/latest/dex/tokens/{}",
        modalresponse.address
    );
    let client = reqwest::Client::new();
    let result = client.get(url).send().await?;
    let parsedresult = result.json::<Root>().await?;
    let basetoken = parsedresult.pairs[0].base_token.clone();

    let mut embed = CreateEmbed::default()
        .title("Check if the following information is correct:")
        .field("Symbol", basetoken.symbol.clone(), false)
        .field("Address", basetoken.address.clone(), false);

    if let Some(logourl) = modalresponse.logo {
        embed = embed.thumbnail(logourl);
    }
    let guildbuttonid = format!("guildbutton-{}", ctx.id());
    let guildbutton = CreateButton::new(guildbuttonid.clone())
        .label("Add token in this server")
        .style(serenity::all::ButtonStyle::Primary);
    let globalbuttonid = format!("globalbutton-{}", ctx.id());
    let globalbutton = CreateButton::new(globalbuttonid.clone())
        .label("Add token in all servers")
        .style(serenity::all::ButtonStyle::Danger);
    let actionrow = CreateActionRow::Buttons(vec![guildbutton.clone(), globalbutton.clone()]);
    let reply = CreateReply::default()
        .embed(embed)
        .components(vec![actionrow])
        .ephemeral(true);
    let replyhandle = ctx.send(reply.clone()).await?;

    let message = match replyhandle.clone().into_message().await {
        Ok(message) => message,
        Err(_) => {
            replyhandle
                .edit(
                    ctx,
                    reply.components(vec![CreateActionRow::Buttons(vec![
                        guildbutton,
                        globalbutton,
                    ])]),
                )
                .await?;
            return Ok(());
        }
    };
    match message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(60 * 2))
        .author_id(ctx.author().id)
        .custom_ids(vec![guildbuttonid.clone(), globalbuttonid.clone()])
        .await
    {
        Some(val) => {
            if !globaltokenpermission && val.data.custom_id.eq(&globalbuttonid) {
                ctx.send(
                    CreateReply::default()
                        .content("This guild does not have the permission to set coins globally"),
                )
                .await?;
                return Ok(());
            }

            let dbresult: Option<Coins> = DB
                .create(("Coins", basetoken.symbol.clone()))
                .content(Coins {
                    name: basetoken.symbol,
                    address: basetoken.address,
                    guildid,
                    global: val.data.custom_id.eq(&globalbuttonid),
                })
                .await?;
            message.delete(ctx).await?;
        }
        None => {
            replyhandle
                .edit(
                    ctx,
                    reply.components(vec![CreateActionRow::Buttons(vec![
                        guildbutton.disabled(true),
                        globalbutton.disabled(true),
                    ])]),
                )
                .await?;
            return Ok(());
        }
    }

    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub schema_version: String,
    pub pairs: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub url: String,
    pub base_token: BaseToken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}
