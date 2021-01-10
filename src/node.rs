use tokio_compat_02::FutureExt;
use web3::contract::{Contract, Options};
use web3::types::U256;

pub async fn df_radius() -> Result<u64, NodeError> {
    let http = web3::transports::Http::new("https://rpc.xdaichain.com")?;
    let web3 = web3::Web3::new(http);

    let contract = Contract::from_json(
        web3.eth(),
        "678ACb78948Be7F354B28DaAb79B1ABD81574c1B".parse()?,
        // todo would be nice to grab the .abi directly from DarkForestCore.json
        include_bytes!("../DarkForest.abi"),
    )?;

    let result = contract.query("worldRadius", (), None, Options::default(), None);
    let world_radius: U256 = result.compat().await?;
    let world_radius: u64 = world_radius.as_u64();

    Ok(world_radius)
}

pub async fn df_players() -> Result<u32, NodeError> {
    let http = web3::transports::Http::new("https://rpc.xdaichain.com")?;
    let web3 = web3::Web3::new(http);

    let contract = Contract::from_json(
        web3.eth(),
        "678ACb78948Be7F354B28DaAb79B1ABD81574c1B".parse()?,
        // todo would be nice to grab the .abi directly from DarkForestCore.json
        include_bytes!("../DarkForest.abi"),
    )?;

    let result = contract.query("getNPlayers", (), None, Options::default(), None);
    let n_players: U256 = result.compat().await?;
    let n_players: u32 = n_players.as_u32();

    Ok(n_players)
}

pub async fn df_counts() -> Result<Vec<u64>, NodeError> {
    let http = web3::transports::Http::new("https://rpc.xdaichain.com")?;
    let web3 = web3::Web3::new(http);

    let contract = Contract::from_json(
        web3.eth(),
        "678ACb78948Be7F354B28DaAb79B1ABD81574c1B".parse()?,
        // todo would be nice to grab the .abi directly from DarkForestCore.json
        include_bytes!("../DarkForest.abi"),
    )?;

    let mut res = vec![];
    let result = contract.query(
        "initializedPlanetCountByLevel",
        (0_u32,),
        None,
        Options::default(),
        None,
    );
    let zero: U256 = result.compat().await?;
    res.push(zero.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (1_u32,),
        None,
        Options::default(),
        None,
    );
    let one: U256 = result.compat().await?;
    res.push(one.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (2_u32,),
        None,
        Options::default(),
        None,
    );
    let two: U256 = result.compat().await?;
    res.push(two.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (3_u32,),
        None,
        Options::default(),
        None,
    );
    let three: U256 = result.compat().await?;
    res.push(three.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (4_u32,),
        None,
        Options::default(),
        None,
    );
    let four: U256 = result.compat().await?;
    res.push(four.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (5_u32,),
        None,
        Options::default(),
        None,
    );
    let five: U256 = result.compat().await?;
    res.push(five.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (6_u32,),
        None,
        Options::default(),
        None,
    );
    let six: U256 = result.compat().await?;
    res.push(six.low_u64());

    let result = contract.query(
        "initializedPlanetCountByLevel",
        (7_u32,),
        None,
        Options::default(),
        None,
    );
    let seven: U256 = result.compat().await?;
    res.push(seven.low_u64());

    Ok(res)
}

#[derive(Debug)]
pub enum NodeError {
    Internal,
    ContractAddress,
    RPCUrl,
    ContractAbi,
    ContractResponseParse,
    JsonError,
    HttpError,
}

impl From<rustc_hex::FromHexError> for NodeError {
    fn from(_err: rustc_hex::FromHexError) -> Self {
        NodeError::ContractAddress
    }
}

impl From<ethabi::Error> for NodeError {
    fn from(_err: ethabi::Error) -> Self {
        NodeError::ContractAbi
    }
}

impl From<web3::Error> for NodeError {
    fn from(_err: web3::Error) -> Self {
        NodeError::RPCUrl
    }
}

impl From<web3::contract::Error> for NodeError {
    fn from(_err: web3::contract::Error) -> Self {
        NodeError::ContractResponseParse
    }
}

impl From<reqwest::Error> for NodeError {
    fn from(_err: reqwest::Error) -> Self {
        NodeError::HttpError
    }
}

impl From<reqwest::header::InvalidHeaderValue> for NodeError {
    fn from(_err: reqwest::header::InvalidHeaderValue) -> Self {
        NodeError::Internal
    }
}

impl From<serde_json::Error> for NodeError {
    fn from(_err: serde_json::Error) -> Self {
        NodeError::JsonError
    }
}
