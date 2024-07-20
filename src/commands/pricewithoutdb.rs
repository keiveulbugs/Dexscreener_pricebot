#![allow(clippy::struct_field_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
use crate::{Context, Data, Error};
use futures::{Stream, StreamExt};
use poise::CreateReply;
use poise::Modal;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
use serenity::model::Colour;
use std::collections::HashMap;

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
// No fucking clue why clippy whines about this
#[allow(clippy::unused_async)]
#[allow(dead_code)] // this code is not dead
async fn autocomplete_name<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let coins = vec!["BTC", "ETH", "OPENX", "OP"];

    futures::stream::iter(coins)
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(std::string::ToString::to_string)
}

/// Find the price of any coin in the Bots database. If not available allow for custom address search.
#[poise::command(slash_command)]
pub async fn price(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[autocomplete = "autocomplete_name"]
    #[description = "Coin to find price from"]
    coin: String,
) -> Result<(), Error> {
    let hardcodedcoins = HashMap::from([
        ("BTC", "0xC0BC84e95864BdfDCd1CCFB8A3AA522E79Ca1410"),
        ("OPENX", "0xc3864f98f2a61A7cAeb95b039D031b4E2f55e0e9"),
        ("OP", "0x4200000000000000000000000000000000000042"),
        ("ETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
    ]);

    let address = match hardcodedcoins.get(coin.to_uppercase().as_str()) {
        Some(coin) => (*coin).to_string(),
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
