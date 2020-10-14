//! `cargo run consumer_key consumer_secret_key access_token secret_access_token`

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::delay_for;
use web3::contract::{Contract, Options};
use web3::futures::TryFutureExt;
use web3::types::U256;

const STAGGER_DELAY: Duration = Duration::from_secs(60 * 60 * 8);

#[tokio::main]
async fn main() {
    let ctrl_c = tokio::signal::ctrl_c().map_err(SophonError::from);

    let _ = futures_micro::or!(ctrl_c, tweets()).await;
}

//ctrlc returns an error so tweets has to in order to match
async fn tweets() -> Result<(), SophonError> {
    let http = web3::transports::Http::new("https://rpc.xdaichain.com")?;
    let web3 = web3::Web3::new(http);

    let contract = Contract::from_json(
        web3.eth(),
        "a8688cCF5E407C1C782CF0c19b3Ab2cE477Fd739".parse()?,
        // todo would be nice to grab the .abi directly from DarkForestCore.json
        include_bytes!("../DarkForest.abi"),
    )?;

    loop {
        let _ = players(contract.clone()).await;
        delay_for(STAGGER_DELAY).await;
        let _ = radius(contract.clone()).await;
        delay_for(STAGGER_DELAY).await;
        let _ = counts(contract.clone()).await;
        delay_for(STAGGER_DELAY).await;
    }
}

async fn radius(contract: Contract<web3::transports::Http>) -> Result<(), SophonError> {
    let result = contract.query("worldRadius", (), None, Options::default(), None);
    let world_radius: U256 = result.await?;
    let world_radius: u64 = world_radius.as_u64();
    dbg!(world_radius);

    let tweet = format!(
        "Sophon 8d9b13c5 TX: the universe has expanded to {} adjust accordingly #darkforest",
        world_radius
    );

    send(&tweet)?;

    Ok(())
}

async fn players(contract: Contract<web3::transports::Http>) -> Result<(), SophonError> {
    let result = contract.query("getNPlayers", (), None, Options::default(), None);
    let n_players: U256 = result.await?;
    let n_players: u32 = n_players.as_u32();
    dbg!(n_players);

    let tweet = format!(
        "Sophon 3a656441 TX: {} civilizations have achieved ftl travel #darkforest",
        n_players
    );

    send(&tweet)?;

    Ok(())
}

async fn counts(contract: Contract<web3::transports::Http>) -> Result<(), SophonError> {
    let result = contract.query("getPlanetCounts", (), None, Options::default(), None);
    let counts: Vec<U256> = result.await?;
    dbg!(counts.clone());

    let tweet = format!(
        "Sophon 02369284 TX: Universe planet totals: {},{},{},{},{},{},{},{} #darkforest",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7]
    );

    send(&tweet)?;

    Ok(())
}

fn send(content: &str) -> Result<(), SophonError> {
    let args: Vec<String> = std::env::args().collect();
    let consumer_key = args[1].clone();
    let consumer_secret_key = args[2].clone();
    let access_token = args[3].clone();
    let secret_access_token = args[4].clone();

    let mut request = url::Url::parse("https://api.twitter.com/1.1/statuses/update.json")?;

    let params: Option<HashMap<&str, &str>> = Some(HashMap::new());
    {
        let mut query_pairs = request.query_pairs_mut();
        query_pairs.append_pair("status", content);
        if let Some(pairs) = params {
            for (key, value) in pairs.iter() {
                query_pairs.append_pair(key, value);
            }
        }
    }

    let url = url::Url::parse(&request.to_string().replace("+", "%20"))?;

    let method = "POST";

    let header = oauthcli::OAuthAuthorizationHeaderBuilder::new(
        method,
        &url,
        consumer_key,
        consumer_secret_key,
        oauthcli::SignatureMethod::HmacSha1,
    )
    .token(access_token, secret_access_token)
    .finish_for_twitter();

    let mut response = reqwest::Client::new()
        .post(&url.to_string())
        .header("Authorization", header.to_string())
        .send()?;

    //todo, TwitterApiError here. but duplicate tweets, like if no planet totals have changed, will error, so just print
    if response.status() != 200 {
        dbg!(response.text().unwrap());
    }
    Ok(())
}

enum SophonError {
    Internal,
    ContractAddress,
    RPCUrl,
    ContractAbi,
    ContractResponseParse,
    TwitterUrl,
    HttpError,
}

impl From<std::io::Error> for SophonError {
    fn from(_err: std::io::Error) -> Self {
        SophonError::Internal
    }
}

impl From<rustc_hex::FromHexError> for SophonError {
    fn from(_err: rustc_hex::FromHexError) -> Self {
        SophonError::ContractAddress
    }
}

impl From<ethabi::Error> for SophonError {
    fn from(_err: ethabi::Error) -> Self {
        SophonError::ContractAbi
    }
}
impl From<web3::Error> for SophonError {
    fn from(_err: web3::Error) -> Self {
        SophonError::RPCUrl
    }
}

impl From<web3::contract::Error> for SophonError {
    fn from(_err: web3::contract::Error) -> Self {
        SophonError::ContractResponseParse
    }
}

impl From<url::ParseError> for SophonError {
    fn from(_err: url::ParseError) -> Self {
        SophonError::TwitterUrl
    }
}

impl From<reqwest::Error> for SophonError {
    fn from(_err: reqwest::Error) -> Self {
        SophonError::HttpError
    }
}
