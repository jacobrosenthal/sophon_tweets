use num_bigint::BigUint;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use tokio_compat_02::FutureExt;

static URL: &str = "https://api.thegraph.com/subgraphs/name/jacobrosenthal/dark-forest-v05";

pub async fn query_graph() -> Result<SophonQueryData, GraphError> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().map_err(GraphError::from)?,
    );

    let body = json!({
        "query": QUERY,
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

    serde_json::from_str::<SophonQueryData>(response.as_str()).map_err(GraphError::from)
}

static QUERY: &str = r#"
query sophon {
    arrivals(where: {processedAt: null}, orderBy: arrivalTime, orderDirection: asc) {
        id
        arrivalId
        arrivalTime
        departureTime
        receivedAt
        energyArriving
        processedAt
        silverMoved
    }
    df_meta: meta(id: 0) {
        lastProcessed
    }
    graph_meta: _meta{
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
pub struct Arrival {
    pub id: String,
    pub arrivalId: u32,
    pub arrivalTime: u32,
    pub departureTime: u32,
    pub receivedAt: u32,
    pub energyArriving: u32,
    pub processedAt: Option<u32>,
    pub silverMoved: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub id: String,
    pub initTimestamp: u32,
    pub homeWorld: Option<Planet>,
    pub planets: Vec<Planet>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Planet {
    pub id: String,
    pub locationDec: BigUint,
    pub isInitialized: bool,
    pub createdAt: u32,
    pub lastUpdated: u32,
    pub perlin: u32,
    pub range: u32,
    pub speed: u32,
    pub defense: u32,
    pub energyLazy: u32,
    pub energyCap: u32,
    pub energyGrowth: u32,
    pub silverCap: u32,
    pub silverGrowth: u32,
    pub silverLazy: u32,
    pub planetLevel: u32,
    pub rangeUpgrades: u32,
    pub speedUpgrades: u32,
    pub defenseUpgrades: u32,
    pub isEnergyCapBoosted: bool,
    pub isSpeedBoosted: bool,
    pub isDefenseBoosted: bool,
    pub isRangeBoosted: bool,
    pub isEnergyGrowthBoosted: bool,
    pub hatLevel: u32,
    pub planetResource: String,
    pub spaceType: String,
    pub silverSpentComputed: u32,
    pub owner: serde_json::Value,
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
