//! `cargo run consumer_key consumer_secret_key access_token secret_access_token`

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use web3::futures::TryFutureExt;

mod graph;
use graph::*;

mod node;
use node::*;

mod twitter;
use twitter::*;

const STAGGER_DELAY: Duration = Duration::from_secs(60 * 60);
const COLLECT_DELAY: Duration = Duration::from_secs(60 * 30);

const RADIUS_DELAY: Duration = Duration::from_secs(60 * 60 * 12);
const COUNTS_DELAY: Duration = Duration::from_secs(60 * 60 * 12);

const STATE_FILE: &str = "sophon_state.json";

#[tokio::main]
async fn main() {
    let state_json = std::fs::read_to_string(STATE_FILE).unwrap_or_default();
    let state = serde_json::from_str::<SophonState>(state_json.as_str()).unwrap_or_default();
    let wrapped_state = Arc::new(Mutex::new(SophonShare { state }));

    let ctrl_c = tokio::signal::ctrl_c().map_err(SophonError::from);

    futures_micro::or!(
        ctrl_c,
        collect_from_graph(wrapped_state.clone()), //COLLECT_DELAY
        collect_from_node(wrapped_state.clone()),  //COLLECT_DELAY
        tweets(wrapped_state.clone()),             //STAGGER_DELAY
        tweet_counts(),                            //COUNTS_DELAY
        tweet_radius()                             //RADIUS_DELAY
    )
    .await
    .unwrap();
}

// ctrlc returns an error so tweets has to in order to match
async fn tweets(wrapped_state: Arc<Mutex<SophonShare>>) -> Result<(), SophonError> {
    loop {
        // scope for mutex release
        {
            let mut share = wrapped_state.lock().await;

            // send a tweet if available
            if let Some(tweet) = share.state.tweets.get(0) {
                // if it sends successfully, pop it to remove it
                if send(tweet.to_owned()).await.is_ok() {
                    share.state.tweets.pop_front();

                    // mutated state so save to disk
                    if let Ok(state_json) = serde_json::to_string(&share.state) {
                        let _ = std::fs::write(STATE_FILE, state_json);
                    }
                }
            }
        }

        sleep(STAGGER_DELAY).await;
    }
}

async fn collect_from_graph(wrapped_state: Arc<Mutex<SophonShare>>) -> Result<(), SophonError> {
    let mut dirty = false;

    loop {
        // scope for mutex release
        {
            let mut share = wrapped_state.lock().await;

            if let Ok(res) = query_graph(share.state.hat_level).await {
                dbg!(res.df_meta.clone());
                if !res.graph_meta.hasIndexingErrors {
                    if let Some(arrival) = res.arrivals.last() {

                        let significant = (arrival.arrivalId / 100000) * 100000;
                        if significant > share.state.significant_arrival {
                            let tweet = format!(
                                "Sophon bacd4f81 TX: {}th departure detected #darkforest",
                                significant
                            );

                            share.state.tweets.push_back(tweet);

                            share.state.significant_arrival = significant;
                            dirty = true;
                        }
                    }

                    if res.arrivals.len() > share.state.most_arrivals_in_motion {
                        let tweet = format!(
                        "Sophon ec1b89f9 TX: Unusually high activity {} movements detected #darkforest",
                        res.arrivals.len()
                    );

                        share.state.tweets.push_back(tweet);

                        share.state.most_arrivals_in_motion = res.arrivals.len();
                        dirty = true;
                    }

                    if !res.hats.is_empty() {
                        let tweet = format!(
                            "Sophon c2463284 TX: {} has discovered lvl {} hat technology #darkforest",
                            res.hats[0].player.id, res.hats[0].hatLevel
                        );

                        share.state.tweets.push_back(tweet);

                        share.state.hat_level = res.hats[0].hatLevel;
                        dirty = true;
                    }

                    for arrival in res.arrivals {
                        let longest_move = arrival.arrivalTime - arrival.departureTime;
                        if longest_move > share.state.longest_move {
                            share.state.longest_move = longest_move;
                        }

                        if arrival.milliSilverMoved > share.state.most_silver_in_motion {
                            let tweet = format!(
                                "Sophon 06cfe9ac TX: Whale alert {} moving {} in silver #darkforest",
                                arrival.player.id,
                                arrival.milliSilverMoved / 1000,
                            );

                            share.state.tweets.push_back(tweet);

                            share.state.most_silver_in_motion = arrival.milliSilverMoved;
                            dirty = true;
                        }
                    }

                    // write out to disc
                    if dirty {
                        if let Ok(state_json) = serde_json::to_string(&share.state) {
                            let _ = std::fs::write(STATE_FILE, state_json);
                        }
                        dirty = false;
                    }
                }
            }
        }
        sleep(COLLECT_DELAY).await;
    }
}

async fn collect_from_node(wrapped_state: Arc<Mutex<SophonShare>>) -> Result<(), SophonError> {
    let mut dirty = false;

    loop {
        // scope for mutex release
        {
            let mut share = wrapped_state.lock().await;

            if let Ok(n_players) = df_players().await {
                dbg!(n_players);

                if n_players > share.state.last_user_count {
                    let tweet = format!(
                        "Sophon 3a656441 TX: {} civilizations have achieved ftl travel #darkforest",
                        n_players
                    );

                    share.state.tweets.push_back(tweet);

                    share.state.last_user_count = n_players;
                    dirty = true;
                }
            }

            if dirty {
                if let Ok(state_json) = serde_json::to_string(&share.state) {
                    let _ = std::fs::write(STATE_FILE, state_json);
                }
                dirty = false;
            }
        }

        sleep(COLLECT_DELAY).await;
    }
}

async fn tweet_counts() -> Result<(), SophonError> {
    loop {
        if let Ok(counts) = df_counts().await {
            dbg!(counts.clone());

            let tweet = format!(
                "Sophon 02369284 TX: Universe planet totals: lvl0:{}, lvl1:{}, lvl2:{}, lvl3:{}, lvl4:{}, lvl5:{}, lvl6:{}, lvl7:{} #darkforest",
                counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7]
            );

            let _ = send(tweet).await;
        }

        sleep(COUNTS_DELAY).await;
    }
}

async fn tweet_radius() -> Result<(), SophonError> {
    loop {
        sleep(RADIUS_DELAY).await;

        if let Ok(world_radius) = df_radius().await {
            dbg!(world_radius);

            let tweet = format!(
                "Sophon 8d9b13c5 TX: the universe has expanded to {} #darkforest",
                world_radius
            );

            let _ = send(tweet).await;
        }
    }
}

pub struct SophonShare {
    state: SophonState,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SophonState {
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
    /// last reported world radius
    last_radius: u64,
    /// biggest hat
    hat_level: u32,
    /// scheduled tweets
    tweets: VecDeque<String>,
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
