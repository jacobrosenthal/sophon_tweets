use num_bigint::BigUint;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use tokio_compat_02::FutureExt;

static URL: &str = "https://api.thegraph.com/subgraphs/name/jacobrosenthal/dark-forest-v05";

#[tokio::main]
async fn main() {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let body = json!({
        "query": QUERY,
    });

    let body = serde_json::to_string(&body).unwrap();

    let res = reqwest::Client::new()
        .post(URL)
        .headers(headers)
        .body(body)
        .send()
        .compat()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    if let Ok(res) = serde_json::from_str::<GraphData>(&res) {
        let mut state = LastState::default();

        println!("{:?}", res.data.arrivals);
        println!("{:?}", res.data.df_meta);
        println!("{:?}", res.data.graph_meta);

        if res.data.arrivals.len() > state.most_arrivals_in_motion {
            state.most_arrivals_in_motion = res.data.arrivals.len();
        }

        // sus
        if res.data.df_meta.lastProcessed % 100000 == 0 {
            // mod 100 000 move
        }
        state.last_processed = res.data.df_meta.lastProcessed;

        for arrival in res.data.arrivals {
            let longest_move = arrival.arrivalTime - arrival.departureTime;
            if longest_move > state.longest_move {
                state.longest_move = longest_move;
            }

            // Whale alert
            if arrival.silverMoved > state.most_silver_in_motion {
                state.most_silver_in_motion = arrival.silverMoved;
            }
        }
    }
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
struct Arrival {
    id: String,
    arrivalId: u32,
    arrivalTime: u32,
    departureTime: u32,
    receivedAt: u32,
    energyArriving: u32,
    processedAt: Option<u32>,
    silverMoved: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Player {
    id: String,
    initTimestamp: u32,
    homeWorld: Option<Planet>,
    planets: Vec<Planet>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Planet {
    id: String,
    locationDec: BigUint,
    isInitialized: bool,
    createdAt: u32,
    lastUpdated: u32,
    perlin: u32,
    range: u32,
    speed: u32,
    defense: u32,
    energyLazy: u32,
    energyCap: u32,
    energyGrowth: u32,
    silverCap: u32,
    silverGrowth: u32,
    silverLazy: u32,
    planetLevel: u32,
    rangeUpgrades: u32,
    speedUpgrades: u32,
    defenseUpgrades: u32,
    isEnergyCapBoosted: bool,
    isSpeedBoosted: bool,
    isDefenseBoosted: bool,
    isRangeBoosted: bool,
    isEnergyGrowthBoosted: bool,
    hatLevel: u32,
    planetResource: String,
    spaceType: String,
    silverSpentComputed: u32,
    owner: serde_json::Value,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GraphMeta {
    hasIndexingErrors: bool,
    deployment: String,
    block: Block,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DarkForestMeta {
    lastProcessed: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    number: u32,
    hash: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GraphData {
    data: SophonQueryData,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SophonQueryData {
    arrivals: Vec<Arrival>,
    graph_meta: GraphMeta,
    df_meta: DarkForestMeta,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct LastState {
    last_processed: u32,            // mod 100 000 move
    most_arrivals_in_motion: usize, // count arrivalsQueues
    longest_move: u32,              // arrivaltime-departuretime
    most_silver_in_motion: u32,     // Whale alert
}
