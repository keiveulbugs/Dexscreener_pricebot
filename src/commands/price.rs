use std::vec;

use crate::Error;


use serenity::utils::Colour;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub pairs: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    #[serde(rename = "chainId")]
    pub chain_id: String,
    #[serde(rename = "dexId")]
    // pub dex_id: String,
    pub url: String,
    #[serde(rename = "pairAddress")]
    pub pair_address: String,
    // #[serde(default)]
    // pub labels: Vec<String>,
    #[serde(rename = "baseToken")]
    pub base_token: BaseToken,
    #[serde(rename = "quoteToken")]
    pub quote_token: QuoteToken,
    #[serde(rename = "priceNative")]
    pub price_native: String,
    #[serde(rename = "priceUsd")]
    pub price_usd: String,
    // pub txns: Txns,
    pub volume: Volume,
    #[serde(rename = "priceChange")]
    pub price_change: PriceChange,
    pub liquidity: Liquidity,
    // #[serde(rename = "pairCreatedAt")]
    // pub pair_created_at: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct Txns {
//     pub h1: H1,
//     pub h24: H24,
//     pub h6: H6,
//     pub m5: M5,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct H1 {
//     pub buys: i64,
//     pub sells: i64,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct H24 {
//     pub buys: i64,
//     pub sells: i64,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct H6 {
//     pub buys: i64,
//     pub sells: i64,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct M5 {
//     pub buys: i64,
//     pub sells: i64,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    pub h24: f64,
    pub h6: f64,
    pub h1: f64,
    pub m5: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceChange {
    pub h24: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Liquidity {
    pub usd: f64,
    pub base: f64,
    pub quote: f64,
}


async fn autocomplete_token(
    _ctx: poise::Context<'_, (), Error>,
    partial: &str,
) -> Vec<String> {
    vec!["openx".to_string(), "optimism".to_string(), "ethereum".to_string()].iter().filter(|token| token.starts_with(partial)).map(|token| token.to_string()).collect()
}


// This is the new price command
#[poise::command(slash_command)]
pub async fn price(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Which token to find the price?"]
    #[autocomplete = "autocomplete_token"]
    name: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let apiresult = reqwest::get(format!(
        "https://api.dexscreener.com/latest/dex/search?q={}",
        name
    ))
    .await
    .map_err(|a| format!("The dexscreener api can not be reached {}", a))?
    .error_for_status()
    .map_err(|_| {
        "This pair can not be retrieved from dexscreener, make sure you write it down correctly"
    })?
    .json::<Root>()
    .await
    .map_err(|b| format!("Something went wrong with parsing the data \n {}", b))?;

    ctx.send(|f| {
        f.embed(|b| {b
            .field(apiresult.pairs[0].base_token.name.clone(), format!("${} : {}%", apiresult.pairs[0].price_usd, apiresult.pairs[0].price_change.h24), true)
            .colour(if apiresult.pairs[0].price_change.h24 > 0.0 {
                Colour::DARK_GREEN
            } else {
                Colour::DARK_RED
            })
        })}).await?;

   // ctx.say(name).await?;
    Ok(())
}