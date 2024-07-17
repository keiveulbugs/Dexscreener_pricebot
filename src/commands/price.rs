#![allow(clippy::struct_field_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
#![cfg(feature = "database")]
use crate::{Context, Data, Error, DB};
use futures::{Stream, StreamExt};
use poise::CreateReply;
use poise::Modal;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
use serenity::model::Colour;

#[derive(Debug, Serialize, Deserialize)]
struct Coins {
    name: String,
    address: String,
}

#[derive(Debug, Modal, Clone)]
#[name = "Custom Token"]
struct CustomToken {
    #[name = "Enter address of the token"] // Field name by default
    #[placeholder = "0x....."] // No placeholder by default
    #[min_length = 42] // No length restriction by default (so, 1-4000 chars)
    #[max_length = 42]
    address: String,
}

async fn autocomplete_name<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let coins: Result<Vec<Coins>, surrealdb::Error> = DB.select("Coins").await;
    let coins = match coins {
        Ok(coins) => coins.iter().map(|x| x.name.clone()).collect(),
        Err(_) => vec![],
    };

    futures::stream::iter(coins)
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

/// Find the price of any coin in the Bots database. If not available allow for custom address search.
#[poise::command(slash_command)]
pub async fn price(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[autocomplete = "autocomplete_name"]
    #[description = "Coin to find price from"]
    coin: String,
) -> Result<(), Error> {
    let optionspecificcoin: Option<Coins> = DB.select(("Coins", coin.clone())).await?;

    let address = match optionspecificcoin {
        Some(coin) => coin.address,
        None => {
            let data = CustomToken::execute(ctx).await?;
            match data {
                Some(val) => val.address,
                None => {
                    ctx.say("This token is not available on dexscreener")
                        .await?;
                    return Ok(());
                }
            }
        }
    };
    ctx.defer().await?;
    let url = format!("https://api.dexscreener.com/latest/dex/tokens/{address}");
    let client = reqwest::Client::new();
    let result = client.get(url).send().await?;
    let mut parsedresult = result.json::<Root>().await?;
    parsedresult.pairs.sort_by_key(|x| {
        x.volume
            .clone()
            .unwrap_or_default()
            .h24
            .unwrap_or(0.0)
            .round() as i64
    });
    let pair = match parsedresult.pairs.last() {
        Some(val) => val,
        None => {
            ctx.say(format!("{} is not available on Dexscreener", coin.clone()))
                .await?;
            return Ok(());
        }
    };
    let price = match &pair.price_usd {
        Some(val) => val,
        None => return Ok(()),
    };
    let pricechange = match &pair.price_change {
        Some(change) => change.h24.unwrap_or(0.0),
        None => return Ok(()),
    };
    let colour = if pricechange >= 0.0 {
        Colour::from_rgb(0, 255, 0)
    } else {
        Colour::from_rgb(255, 0, 0)
    };
    let nametoken = &pair.base_token.name;

    let embed = CreateEmbed::default()
        .author(CreateEmbedAuthor::new(nametoken))
        .title(format!("${price}    *( {pricechange}%)*"))
        .footer(CreateEmbedFooter::new(
            "All rights reserved to Dexscreener.com",
        ))
        .colour(colour);

    ctx.send(CreateReply::default().embed(embed)).await?;

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
    pub price_usd: Option<String>,
    pub volume: Option<Volume>,
    pub price_change: Option<PriceChange>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub h24: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange {
    pub h24: Option<f64>,
}
