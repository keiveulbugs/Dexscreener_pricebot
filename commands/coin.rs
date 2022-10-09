use crate::Error;
use serde_derive::Deserialize;
use serde_json::Value;
//use poise::serenity_prelude as serenit;
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
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Name {
    pub name: String,
}

pub struct Resultstruct {
    pub name: String,
    pub usd: f64,
    pub change: String,
    pub volume: String,
    pub colour: Colour,
    pub urlresult: String,
}

#[derive(poise::ChoiceParameter)]
#[allow(non_camel_case_types)]
pub enum Coin {
    Openx,
    XopenX,
    OP,
    Eth,
}

/// Get info on a coin by entering their symbol
#[poise::command(slash_command)]
pub async fn coin(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Select a Coin or Token"] coin: Coin,
) -> Result<(), Error> {
    let response = match coin {
        Coin::Openx => vectorinfoinverse("https://api.dexscreener.com/latest/dex/pairs/optimism/0x442659a6d04b907c879032da1ef634548110dd37").await?,
        Coin::XopenX => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0x7ed0ac1dced6da79369ba36c5f48679f2d4daa90").await?,
        Coin::OP => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0x47029bc8f5cbe3b464004e87ef9c9419a48018cd").await?,
        Coin::Eth => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0x85149247691df622eaf1a8bd0cafd40bc45154a9").await?,
    };

    ctx.send(|b| {
        b.embed(|b| {
            b.description(format!(
                "Price : ${:.4}\nVolume : ${}\nChange : {}",
                response.usd, response.volume, response.change
            ))
            .title(response.name.to_string())
            .colour(response.colour)
            .url(&response.urlresult)
        })
        .ephemeral(false)
    })
    .await?;

    //ctx.say(response).await?;
    Ok(())
}

async fn vectorinfo(url: &str) -> Result<Resultstruct, Error> {
    let v = reqwest::get(url)
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
    let usd0 = w[0].price_usd.parse::<f64>().unwrap();
    let volume = &w[0].volume["h24"].to_string();
    let name0 = &w[0].base_token.name;
    let change0 = w[0].price_change.h24;
    let changestring = format!("{}%", change0);
    let colour1 = if change0 > 0.0 {
        Colour::DARK_GREEN
    } else if change0 < 0.0 {
        Colour::RED
    } else {
        Colour::GOLD
    };

    let finalstruct = Resultstruct {
        name: name0.to_string(),
        usd: usd0,
        change: changestring,
        volume: volume.to_string(),
        colour: colour1,
        urlresult: w[0].url.to_string(),
    };
    Ok(finalstruct)
}

async fn vectorinfoinverse(url: &str) -> Result<Resultstruct, Error> {
    let v = reqwest::get(url)
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
    let price0 = w[0].price_native.parse::<f64>().unwrap();
    let usd0 = w[0].price_usd.parse::<f64>().unwrap();
    let usd1 = usd0 / price0;
    let name1 = &w[0].quote_token.name;
    let volume = &w[0].volume["h24"].to_string();
    let colour1 = Colour::GOLD;
    let changestring = "No data available".to_string();
    let finalstruct = Resultstruct {
        name: name1.to_string(),
        usd: usd1,
        volume: volume.to_string(),
        change: changestring,
        colour: colour1,
        urlresult: w[0].url.to_string(),
    };
    Ok(finalstruct)
}
