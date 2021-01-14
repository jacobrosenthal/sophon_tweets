// use num_bigint::BigUint;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use tokio_compat_02::FutureExt;

static URL: &str = "https://api.thegraph.com/subgraphs/name/jacobrosenthal/dark-forest-v05";

pub async fn query_graph(hat_level: u32, planet_level: u32) -> Result<SophonQueryData, GraphError> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().map_err(GraphError::from)?,
    );

    let body = json!({
        "query": QUERY,
        "variables": {
            "hat_level": hat_level,
            "planet_level": planet_level
        }
    });

    let body = serde_json::to_string(&body).map_err(GraphError::from)?;

    let response = reqwest::Client::new()
        .post(URL)
        .headers(headers)
        .body(body)
        .send()
        .compat()
        .await
        .map_err(GraphError::from)?
        .text()
        .await
        .map_err(GraphError::from)?;

    let sophon_result =
        serde_json::from_str::<GraphData>(response.as_str()).map_err(GraphError::from)?;

    Ok(sophon_result.data)
}

static QUERY: &str = r#"
query sophon($hat_level: Int!, $planet_level: Int!) {
    arrivals(where: {processedAt: null}, orderBy: arrivalTime, orderDirection: asc) {
        id
        arrivalId
        arrivalTime
        departureTime
        receivedAt
        milliEnergyArriving
        processedAt
        milliSilverMoved
        player {
          id
          initTimestamp
        }
        fromPlanet{
          id
          speed
        }
    }
    hats(first: 1, where: {hatLevel_gt: $hat_level}, orderBy:hatLevel, orderDirection:asc) {
        id
        hatLevel
        planet {
          id
          speed
        }
        player {
          id
          initTimestamp
        }
        timestamp
    }
    artifacts( first: 1, where: {planetLevel_gt: $planet_level}, orderBy:artifactId, orderDirection:asc) {
        id
        rarity
    	planetLevel
        discoverer {
          id
          initTimestamp
        }
        planetDiscoveredOn{
          id
          speed
        }
    }
    df_meta: meta(id: 0) {
        lastProcessed
    }
    graph_meta: _meta {
        deployment
        hasIndexingErrors
        block{
            number
            hash
        }
    }
}
"#;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub planetLevel: u32,
    pub rarity: String,
    pub discoverer: Player,
    pub planetDiscoveredOn: Planet,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arrival {
    pub id: String,
    pub arrivalId: u32,
    pub arrivalTime: u32,
    pub departureTime: u32,
    pub receivedAt: u32,
    pub milliEnergyArriving: u32,
    pub processedAt: Option<u32>,
    pub milliSilverMoved: u32,
    pub fromPlanet: Planet,
    pub player: Player,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub id: String,
    pub initTimestamp: u32,
    // pub homeWorld: Option<Planet>,
    // pub planets: Vec<Planet>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hat {
    pub id: String,
    pub planet: Planet,
    pub player: Player,
    pub hatLevel: u32,
    pub timestamp: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Planet {
    pub id: String,
    // pub locationDec: BigUint,
    // pub isInitialized: bool,
    // pub createdAt: u32,
    // pub lastUpdated: u32,
    // pub perlin: u32,
    // pub range: u32,
    pub speed: u32,
    // pub defense: u32,
    // pub milliEnergyLazy: u32,
    // pub milliEnergyCap: u32,
    // pub milliEnergyGrowth: u32,
    // pub milliSilverCap: u32,
    // pub milliSilverGrowth: u32,
    // pub milliSilverLazy: u32,
    // pub planetLevel: u32,
    // pub rangeUpgrades: u32,
    // pub speedUpgrades: u32,
    // pub defenseUpgrades: u32,
    // pub ismilliEnergyCapBoosted: bool,
    // pub isSpeedBoosted: bool,
    // pub isDefenseBoosted: bool,
    // pub isRangeBoosted: bool,
    // pub ismilliEnergyGrowthBoosted: bool,
    // pub hatLevel: u32,
    // pub planetResource: String,
    // pub spaceType: String,
    // pub milliSilverSpent: u32,
    // pub owner: serde_json::Value,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphMeta {
    pub hasIndexingErrors: bool,
    pub deployment: String,
    pub block: Block,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DarkForestMeta {
    pub lastProcessed: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub number: u32,
    pub hash: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphData {
    pub data: SophonQueryData,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SophonQueryData {
    pub arrivals: Vec<Arrival>,
    pub graph_meta: GraphMeta,
    pub df_meta: DarkForestMeta,
    pub hats: Vec<Hat>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug)]
pub enum GraphError {
    Internal,
    JsonError,
    HttpError,
}

impl From<reqwest::Error> for GraphError {
    fn from(_err: reqwest::Error) -> Self {
        GraphError::HttpError
    }
}

impl From<reqwest::header::InvalidHeaderValue> for GraphError {
    fn from(_err: reqwest::header::InvalidHeaderValue) -> Self {
        GraphError::Internal
    }
}

impl From<serde_json::Error> for GraphError {
    fn from(_err: serde_json::Error) -> Self {
        GraphError::JsonError
    }
}
