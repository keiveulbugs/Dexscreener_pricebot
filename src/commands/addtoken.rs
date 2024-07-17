use crate::{Context, Data, Error, DB};
use poise::{CreateReply, Modal};
use serde::{Deserialize, Serialize};
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

pub async fn addtoken(ctx: Context<'_>, interaction: ComponentInteraction) -> Result<(), Error> {
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
        .field("Symbol", basetoken.symbol, false)
        .field("Address", basetoken.address, false);

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
    let actionrow = CreateActionRow::Buttons(vec![guildbutton, globalbutton]);
    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![actionrow])
            .ephemeral(true),
    )
    .await?;

    while let Some(mci) = poise::serenity_prelude::ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        // .filter(move |mci| {
        //     mci.data.custom_id == guildbuttonid.to_string()
        //         || mci.data.custom_id == globalbuttonid.to_string()
        // })
        .await
    {
        let mut msg = mci.message.clone();

        mci.create_response(
            ctx,
            poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
        )
        .await?;
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
