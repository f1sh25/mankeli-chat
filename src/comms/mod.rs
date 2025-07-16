//config.json has settings to set fetch frequency

// Message Fetcher
// loops trough accepted friend list and fetches new messages
// updates last seen which is a time delta 5 min ago

// Friend Fetcher
// gets list of undelivered requests from table
// if success then set sent flag to true

use crate::StatusLabel;
use crate::api::{FetchMessageInput, FriendInput, FriendRequestStatus};
use crate::db::{
    Friend, Message, batch_ingest, fetch_active_friends, fetch_unsent_friend_updt,
    update_friend_status_as_sent,
};
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;

pub async fn message_fetcher(pool: &SqlitePool, our_username: &str, our_address: &str) {
    let client = Client::new();
    println!("Message fetcher started.");

    loop {
        // 1. Fetch the list of friends from the database.
        let friend_list = match fetch_active_friends(pool).await {
            Ok(friends) => friends,
            Err(e) => {
                // If the DB query fails, log it and wait before retrying.
                eprintln!("Error fetching friend list: {}. Retrying in 60s.", e);
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue; // Skip to the next loop iteration.
            }
        };

        if friend_list.is_empty() {
            println!("No active friends found. Checking again in 30s.");
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        }

        // 2. Process each friend concurrently.

        const CONCURRENT_REQUESTS: usize = 10;

        stream::iter(friend_list)
            .for_each_concurrent(CONCURRENT_REQUESTS, |friend| {
                let client = client.clone();
                let pool = pool.clone();
                // Provide the info the remote server needs to identify us.
                let req_body = FetchMessageInput {
                    username: our_username.to_string(),
                    address: our_address.to_string(),
                };

                async move {
                    let target_url = format!("http://{}/fetch_messages", friend.address);
                    println!(
                        "Fetching messages from {} at {}",
                        friend.username, target_url
                    );

                    let response = client.post(&target_url).json(&req_body).send().await;

                    match response {
                        Ok(res) => {
                            // Check for non-successful status codes (e.g., 404, 500).
                            if !res.status().is_success() {
                                eprintln!(
                                    "Request to {} failed with status: {}",
                                    friend.username,
                                    res.status()
                                );
                                return;
                            }

                            // 3. Attempt to parse the response and ingest messages.
                            match res.json::<Vec<Message>>().await {
                                Ok(messages) if !messages.is_empty() => {
                                    if let Err(e) = batch_ingest(&pool, messages).await {
                                        eprintln!(
                                            "Failed to ingest messages from {}: {}",
                                            friend.username, e
                                        );
                                    }
                                }
                                Ok(_) => { /* No new messages, do nothing. */ }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse JSON from {}: {}",
                                        friend.username, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to send request to {}: {}", friend.username, e);
                        }
                    }
                }
            })
            .await;

        // 4. Wait before starting the next cycle.
        println!("Fetch cycle complete. Waiting for 30s.");
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

pub async fn friend_fetcher(pool: &SqlitePool) {
    let client = Client::new();
    println!("Friend fetcher service started.");

    loop {
        let (our_username, friend_list) = match fetch_unsent_friend_updt(pool).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("DB Error fetching friend updates: {}. Retrying in 60s.", e);
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        if friend_list.is_empty() {
            tokio::time::sleep(Duration::from_secs(15)).await;
            continue;
        }

        const CONCURRENT_REQUESTS: usize = 5;

        stream::iter(friend_list)
            .for_each_concurrent(CONCURRENT_REQUESTS, |friend| {
                let client = client.clone();
                let pool = pool.clone();
                let our_username = our_username.clone();

                async move {
                    let req_body = FriendInput {
                        username: friend.username.clone(),
                        hostname: our_username,
                        req_type: friend.status.status_enum(),
                    };

                    let target_url = format!("http://{}/friend_request", friend.address);
                    println!(
                        "Sending friend request to {} at {}",
                        friend.username, target_url
                    );

                    match client.post(&target_url).json(&req_body).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                println!("Successfully sent friend request to {}", friend.username);
                                if let Err(e) =
                                    update_friend_status_as_sent(&pool, &friend.username).await
                                {
                                    eprintln!(
                                        "DB Error updating status for {}: {}",
                                        friend.username, e
                                    );
                                }
                            } else {
                                eprintln!(
                                    "Failed request to {}. Status: {}",
                                    friend.username,
                                    response.status()
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Network error sending request to {}: {}",
                                friend.username, e
                            );
                        }
                    }
                }
            })
            .await;

        println!("Friend update cycle complete. Waiting for 15s.");
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}
