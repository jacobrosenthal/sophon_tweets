//! `cargo run consumer_key consumer_secret_key access_token secret_access_token`

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tokio::time::sleep;
use web3::futures::TryFutureExt;

mod graph;
use graph::*;

mod node;
use node::*;

mod twitter;
use twitter::*;

const STAGGER_DELAY: Duration = Duration::from_secs(60 * 60);
const COUNTS_DELAY: Duration = Duration::from_secs(60 * 45);
const COLLECT_DELAY: Duration = Duration::from_secs(60 * 30);

const GRAPH_FILE: &str = "graph_state.json";
const NODE_FILE: &str = "node_state.json";

#[tokio::main]
async fn main() {
    let (sender, receiver) = channel::<String>();

    let ctrl_c = tokio::signal::ctrl_c().map_err(SophonError::from);

    futures_micro::or!(
        ctrl_c,
        tweets(receiver),                   //STAGGER_DELAY
        collect_from_graph(sender.clone()), //COLLECT_DELAY
        collect_from_node(sender.clone()),  //COLLECT_DELAY
        tweet_counts()                      //COUNTS_DELAY
    )
    .await
    .unwrap();
}

// ctrlc returns an error so tweets has to in order to match
async fn tweets(chan: Receiver<String>) -> Result<(), SophonError> {
    let mut tweets: VecDeque<String> = VecDeque::new();

    loop {
        // drain the channel
        while let Ok(tweet) = chan.recv() {
            tweets.push_back(tweet);
        }

        // send a tweet if available
        if let Some(tweet) = tweets.pop_front() {
            let _ = send(tweet).await;
        }

        sleep(STAGGER_DELAY).await;
    }
}

async fn collect_from_graph(chan: Sender<String>) -> Result<(), SophonError> {
    let state_json = std::fs::read_to_string(GRAPH_FILE).unwrap_or_default();
    let mut state = serde_json::from_str::<GraphState>(state_json.as_str()).unwrap_or_default();
    let mut dirty = false;
    loop {
        if let Ok(res) = query_graph().await {
            dbg!(res.arrivals.clone());
            dbg!(res.df_meta.clone());
            dbg!(res.graph_meta.clone());

            let significant = ((res.df_meta.lastProcessed % 100000) + 100000) % 100000;
            if significant > state.significant_arrival {
                let tweet = format!(
                    "Sophon bacd4f81 TX: {}th departure detected #darkforest",
                    significant
                );

                let _ = chan.send(tweet);

                state.significant_arrival = significant;
                dirty = true;
            }

            if res.arrivals.len() > state.most_arrivals_in_motion {
                let tweet = format!(
                    "Sophon ec1b89f9 TX: Unusually high activity {} movements detected #darkforest",
                    res.arrivals.len()
                );

                let _ = chan.send(tweet);

                state.most_arrivals_in_motion = res.arrivals.len();
                dirty = true;
            }

            for arrival in res.arrivals {
                let longest_move = arrival.arrivalTime - arrival.departureTime;
                if longest_move > state.longest_move {
                    state.longest_move = longest_move;
                }

                if arrival.milliSilverMoved > state.most_silver_in_motion {
                    let tweet = format!(
                        "Sophon 06cfe9ac TX: Whale alert {} silver in motion #darkforest",
                        res.df_meta.lastProcessed % 100000
                    );

                    let _ = chan.send(tweet);

                    state.most_silver_in_motion = arrival.milliSilverMoved;
                    dirty = true;
                }
            }

            // write out to disc
            if dirty {
                if let Ok(state_json) = serde_json::to_string(&state) {
                    let _ = std::fs::write(GRAPH_FILE, state_json);
                }
                dirty = false;
            }
            sleep(COLLECT_DELAY).await;
        }
    }
}

async fn collect_from_node(chan: Sender<String>) -> Result<(), SophonError> {
    let state_json = std::fs::read_to_string(NODE_FILE).unwrap_or_default();
    let mut state = serde_json::from_str::<NodeState>(state_json.as_str()).unwrap_or_default();
    let mut dirty = false;

    loop {
        if let Ok(world_radius) = df_radius().await {
            dbg!(world_radius);

            if world_radius > state.last_radius {
                let tweet = format!(
                    "Sophon 8d9b13c5 TX: the universe has expanded to {} adjust accordingly #darkforest",
                    world_radius
                );

                let _ = chan.send(tweet);

                state.last_radius = world_radius;
                dirty = true;
            }
        }

        if let Ok(n_players) = df_players().await {
            dbg!(n_players);

            if n_players > state.last_user_count {
                let tweet = format!(
                    "Sophon 3a656441 TX: {} civilizations have achieved ftl travel #darkforest",
                    n_players
                );

                let _ = chan.send(tweet);

                state.last_user_count = n_players;
                dirty = true;
            }
        }

        if dirty {
            if let Ok(state_json) = serde_json::to_string(&state) {
                let _ = std::fs::write(NODE_FILE, state_json);
            }
            dirty = false;
        }

        sleep(COLLECT_DELAY).await;
    }
}

async fn tweet_counts() -> Result<(), SophonError> {
    loop {
        sleep(COUNTS_DELAY).await;

        if let Ok(counts) = df_counts().await {
            dbg!(counts.clone());

            let tweet = format!(
                "Sophon 02369284 TX: Universe planet totals: lvl0:{}, lvl1:{}, lvl2:{}, lvl3:{}, lvl4:{}, lvl5:{}, lvl6:{}, lvl7:{} #darkforest",
                counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7]
            );

            let _ = send(tweet).await;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GraphState {
    /// count of unprocessed arrivalsQueues
    most_arrivals_in_motion: usize,
    /// n hundred thousandth arrival
    significant_arrival: u32,
    /// arrivaltime-departuretime in seconds
    longest_move: u32,
    /// Whale alert in millisilver
    most_silver_in_motion: u32,
    /// how many users in system
    last_user_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeState {
    /// how many users in system
    last_user_count: u32,
    /// last reported world radius
    last_radius: u64,
}

#[derive(Debug)]
enum SophonError {
    Internal,
}

impl From<std::io::Error> for SophonError {
    fn from(_err: std::io::Error) -> Self {
        SophonError::Internal
    }
}
