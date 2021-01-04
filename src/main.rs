//! `cargo run consumer_key consumer_secret_key access_token secret_access_token`

use reqwest::multipart;
use reqwest_oauth1::OAuthClientProvider;
use std::time::Duration;
use tokio::time::sleep;
use web3::contract::{Contract, Options};
use web3::futures::TryFutureExt;
use web3::types::U256;

const STAGGER_DELAY: Duration = Duration::from_secs(60 * 60 * 2);

#[tokio::main]
async fn main() {
    let ctrl_c = tokio::signal::ctrl_c().map_err(SophonError::from);

    futures_micro::or!(ctrl_c, tweets()).await.unwrap();
}

//ctrlc returns an error so tweets has to in order to match
async fn tweets() -> Result<(), SophonError> {
    let http = web3::transports::Http::new("https://rpc.xdaichain.com")?;
    let web3 = web3::Web3::new(http);

    let contract = Contract::from_json(
        web3.eth(),
        "678ACb78948Be7F354B28DaAb79B1ABD81574c1B".parse()?,
        // todo would be nice to grab the .abi directly from DarkForestCore.json
        include_bytes!("../DarkForest.abi"),
    )?;

    loop {
        let _ = counts(contract.clone()).await;
        sleep(STAGGER_DELAY).await;
        let _ = players(contract.clone()).await;
        sleep(STAGGER_DELAY).await;
        let _ = radius(contract.clone()).await;
        sleep(STAGGER_DELAY).await;
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

    send(tweet).await?;

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

    send(tweet).await?;

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

    send(tweet).await?;

    Ok(())
}

async fn send(tweet: String) -> Result<(), SophonError> {
    let args: Vec<String> = std::env::args().collect();
    let consumer_key = args[1].clone();
    let consumer_secret_key = args[2].clone();
    let access_token = args[3].clone();
    let secret_access_token = args[4].clone();

    let secrets = reqwest_oauth1::Secrets::new(consumer_key, consumer_secret_key)
        .token(access_token, secret_access_token);

    let endpoint = "https://api.twitter.com/1.1/statuses/update.json";

    let content = multipart::Form::new().text("status", tweet);

    dbg!("bfore");

    let response = reqwest::Client::new()
        // enable OAuth1 request
        .oauth1(secrets)
        .post(endpoint)
        .multipart(content)
        .send()
        .await
        .unwrap();

    dbg!("after");

    //todo, TwitterApiError here. but duplicate tweets, like if no planet totals have changed, will error, so just print
    if response.status() != 200 {
        dbg!(response.text().await.unwrap());
    }
    Ok(())
}

#[derive(Debug)]
enum SophonError {
    Internal,
    ContractAddress,
    RPCUrl,
    ContractAbi,
    ContractResponseParse,
    TwitterUrl,
    HttpError,
    OAuth,
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

impl From<reqwest_oauth1::Error> for SophonError {
    fn from(_err: reqwest_oauth1::Error) -> Self {
        SophonError::OAuth
    }
}
