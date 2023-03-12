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
    pub symbol: String,
}

pub struct Resultstruct {
    pub name: String,
    pub usd: String,
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
    OpxveVelo,
    Waru,
}

/// Get info on a coin by entering their symbol
#[poise::command(slash_command)]
pub async fn coin(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Select a Coin or Token"] coin: Coin,
) -> Result<(), Error> {
    let response = match coin {
        Coin::Openx => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0x23b06d0b8a6c267d8af404cea067f06fa3c6012d").await?,
        Coin::XopenX => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0x7ed0ac1dced6da79369ba36c5f48679f2d4daa90").await?,
        Coin::OP => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0xcdd41009e74bd1ae4f7b2eecf892e4bc718b9302").await?,
        Coin::Eth => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/optimism/0xc858a329bf053be78d6239c4a4343b8fbd21472b").await?,
        Coin::OpxveVelo => vectorinfoinverseinbase("https://api.dexscreener.com/latest/dex/pairs/optimism/0x946021c382de2aba5c7aba3cb00e67c6e0ffa787").await?,
        Coin::Waru => vectorinfo("https://api.dexscreener.com/latest/dex/pairs/polygon/0x287b27b3008037615e5fe3cacc867c1f9b39c24700020000000000000000083a-0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270-0xe3627374ac4baf5375e79251b0af23afc450fc0e").await?,
    };

    ctx.send(|b| {
        b.embed(|b| {
            b.field("Price", response.usd, true)
                .field("Volume", format!("${}", response.volume), true)
                .field("Change", response.change, true)
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
    let usd0 = format!("${:.4}", w[0].price_usd.parse::<f64>().unwrap());
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

#[allow(dead_code)]
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
        usd: format!("${:.4}", usd1),
        volume: volume.to_string(),
        change: changestring,
        colour: colour1,
        urlresult: w[0].url.to_string(),
    };
    Ok(finalstruct)
}

async fn vectorinfoinverseinbase(url: &str) -> Result<Resultstruct, Error> {
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
    //let usd0 = w[0].price_usd.parse::<f64>().unwrap();
    //let usd1 = usd0 / price0;
    let name1 = &w[0].quote_token.name;
    let volume = &w[0].volume["h24"].to_string();
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
        name: name1.to_string(),
        usd: format!(
            "{} : {} {}",
            &w[0].base_token.symbol, price0, &w[0].quote_token.symbol
        ),
        volume: volume.to_string(),
        change: changestring,
        colour: colour1,
        urlresult: w[0].url.to_string(),
    };
    Ok(finalstruct)
}
