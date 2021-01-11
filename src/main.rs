//! `cargo run consumer_key consumer_secret_key access_token secret_access_token`

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use web3::futures::TryFutureExt;

mod graph;
use graph::*;

mod node;
use node::*;

mod twitter;
use twitter::*;

const STAGGER_DELAY: Duration = Duration::from_secs(60 * 60 * 2);

#[tokio::main]
async fn main() {
    let ctrl_c = tokio::signal::ctrl_c().map_err(SophonError::from);

    futures_micro::or!(ctrl_c, tweets()).await.unwrap();
}

async fn graph() -> Result<(), SophonError> {
    let res = query_graph().await.unwrap();

    let mut state = LastState::default();

    println!("{:?}", res.arrivals);
    println!("{:?}", res.df_meta);
    println!("{:?}", res.graph_meta);

    let significant = ((res.df_meta.lastProcessed % 100000) + 100000) % 100000;
    if significant > state.significant_arrival {
        let tweet = format!(
            "Sophon bacd4f81 TX: {}th departure detected #darkforest",
            significant
        );

        state.tweets.push(tweet);

        state.significant_arrival = significant;
    }

    if res.arrivals.len() > state.most_arrivals_in_motion {
        let tweet = format!(
            "Sophon ec1b89f9 TX: Unusually high activity {} in motion #darkforest",
            res.arrivals.len()
        );

        state.tweets.push(tweet);

        state.most_arrivals_in_motion = res.arrivals.len();
    }

    for arrival in res.arrivals {
        let longest_move = arrival.arrivalTime - arrival.departureTime;
        if longest_move > state.longest_move {
            state.longest_move = longest_move;
        }

        // Whale alert
        if arrival.silverMoved > state.most_silver_in_motion {
            let tweet = format!(
                "Sophon 06cfe9ac TX: Whale alert {} silver in motion #darkforest",
                res.df_meta.lastProcessed % 100000
            );

            state.tweets.push(tweet);

            state.most_silver_in_motion = arrival.silverMoved;
        }
    }

    Ok(())
}

//ctrlc returns an error so tweets has to in order to match
async fn tweets() -> Result<(), SophonError> {
    loop {
        let _ = counts().await;
        sleep(STAGGER_DELAY).await;
        let _ = players().await;
        sleep(STAGGER_DELAY).await;
        let _ = radius().await;
        sleep(STAGGER_DELAY).await;
        let _ = graph().await;
        sleep(STAGGER_DELAY).await;
    }
}

async fn radius() -> Result<(), TwitterError> {
    let world_radius = df_radius().await.unwrap();
    dbg!(world_radius);

    let tweet = format!(
        "Sophon 8d9b13c5 TX: the universe has expanded to {} adjust accordingly #darkforest",
        world_radius
    );

    send(tweet).await.unwrap();

    Ok(())
}

async fn players() -> Result<(), TwitterError> {
    let n_players = df_players().await.unwrap();
    dbg!(n_players);

    let tweet = format!(
        "Sophon 3a656441 TX: {} civilizations have achieved ftl travel #darkforest",
        n_players
    );

    send(tweet).await.unwrap();

    Ok(())
}

async fn counts() -> Result<(), TwitterError> {
    let counts = df_counts().await.unwrap();
    dbg!(counts.clone());

    let tweet = format!(
        "Sophon 02369284 TX: Universe planet totals: lvl0:{}, lvl1:{}, lvl2:{}, lvl3:{}, lvl4:{}, lvl5:{}, lvl6:{}, lvl7:{} #darkforest",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7]
    );

    send(tweet).await.unwrap();

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LastState {
    /// count of unprocessed arrivalsQueues
    most_arrivals_in_motion: usize,
    /// n hundred thousandth arrival
    significant_arrival: u32,
    /// arrivaltime-departuretime
    longest_move: u32,
    /// Whale alert
    most_silver_in_motion: u32,
    /// tweets scheduled to go out
    tweets: Vec<String>,
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
