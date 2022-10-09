use crate::Error;
use serde_derive::Deserialize;
//use serde_derive::Serialize;
use serde_json::Value;
use serenity::utils::Colour;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct L1 {
    pub pairs: Vec<L2>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L2 {
    pub chain_id: String,
    pub dex_id: String,
    pub url: String,
    pub pair_address: String,
    pub price_native: String,
    pub price_usd: String,
    pub price_change: Change,
    pub liquidity: Value,
    pub volume: Value,
    pub base_token: Name,
    pub quote_token: Name,
    //pub fdv: f64,
}
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Change {
    pub h24: f64,
    pub h6: f64,
    pub h1: f64,
    pub m5: f64,
}
#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Name {
    pub name: String,
}

#[derive(poise::ChoiceParameter)]
#[allow(non_camel_case_types)]
pub enum Chain {
    ethereum,
    #[name = "binance_smart_chain"]
    bsc,
    polygon,
    avalanche,
    fantom,
    harmony,
    cronos,
    aurora,
    moonriver,
    moonbeam,
    metis,
    arbitrum,
    optimism,
    dogechain,
    heco,
    astar,
    evmos,
    xdai,
}

/// Get info on a coin by entering their symbol
#[poise::command(slash_command)]
pub async fn address_search(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Select a chain"] chain: Chain,
    #[description = "Enter a symbol, or choose a default one!"] address: String,
    #[description = "turn this on if you want to invert the pair"] invert: bool,
) -> Result<(), Error> {
    let v = reqwest::get(format!(
        "https://api.dexscreener.com/latest/dex/pairs/{}/{}",
        chain, address
    ))
    .await
    .map_err(|_| "The dexscreener api can not be reached")?
    .error_for_status()
    .map_err(|_| {
        "This pair can not be retrieved from dexscreener, make sure you write it down correctly"
    })?
    .json::<L1>()
    .await
    .map_err(|_| "Something went wrong with parsing the data")?;
    let w = v.pairs;
    if invert {
        let price0 = w[0].price_native.parse::<f64>().unwrap();
        let usd0 = w[0].price_usd.parse::<f64>().unwrap();
        let usd1 = usd0 / price0;
        let name1 = &w[0].quote_token.name;
        let volume = &w[0].volume["h24"].to_string();
        let urlresult = &w[0].url;
        ctx.send(|b| {
            b.embed(|b| {
                b.description(format!(
                    "\nPrice : ${:.4}\nVolume : ${}\nChange : No data available",
                    usd1, volume
                ))
                .title(name1)
                .colour(Colour::GOLD)
                .url(urlresult)
            })
            .ephemeral(false)
        })
        .await?;
    } else {
        let usd0 = w[0].price_usd.parse::<f64>().unwrap();
        let volume = &w[0].volume["h24"];
        let name0 = &w[0].base_token.name;
        let change0 = w[0].price_change.h24;
        let urlresult = &w[0].url;
        let colour1 = if change0 > 0.0 {
            Colour::DARK_GREEN
        } else if change0 < 0.0 {
            Colour::RED
        } else {
            Colour::GOLD
        };
        ctx.send(|b| {
            b.embed(|b| {
                b.description(format!(
                    "Price : ${:.4}\nVolume : ${}\nChange : {}%",
                    usd0, volume, change0
                ))
                .title(name0)
                .colour(colour1)
                .url(urlresult)
            })
            .ephemeral(false)
        })
        .await?;
    };
    Ok(())
}
