//config.json has settings to set fetch frequency

// Message Fetcher
// loops trough accepted friend list and fetches new messages
// updates last seen which is a time delta 5 min ago

// Friend Fetcher
// gets list of undelivered requests from table
// if success then set sent flag to true

use crate::StatusLabel;
use crate::api::{FetchMessageInput, FetchMessageResponse, FriendInput};
use crate::db::{
    Friend, batch_ingest, fetch_active_friends, fetch_unsent_friend_updt,
    update_friend_status_as_sent,
};
use futures::stream::{self, StreamExt};
use reqwest::Client;
use sqlx::SqlitePool;
use std::time::Duration;

#[cfg(test)]
mod tests;

pub async fn process_friend_messages(
    pool: &SqlitePool,
    client: &Client,
    our_username: &str,
    our_address: &str,
    friend: &Friend,
) -> Result<(), String> {
    let target_url = format!("http://{}/fetch_messages", friend.address);

    let req_body = FetchMessageInput {
        username: our_username.to_string(),
        address: our_address.to_string(),
    };

    let res = client
        .post(&target_url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Bad status: {}", res.status()));
    }

    let apiresponse = res
        .json::<FetchMessageResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    if !apiresponse.messages.is_empty() {
        batch_ingest(pool, apiresponse.messages)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    Ok(())
}

pub async fn message_fetcher(
    pool: &SqlitePool,
    our_username: &str,
    our_address: &str,
    sleep_time: u64,
) {
    let client = Client::new();
    println!("Message fetcher started.");

    loop {
        let friend_list = match fetch_active_friends(pool).await {
            Ok(friends) => friends,
            Err(e) => {
                eprintln!("Error fetching friend list: {}. Retrying in 60s.", e);
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        if friend_list.is_empty() {
            //println!("No active friends found. Checking again in 30s.");
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        }

        const CONCURRENT_REQUESTS: usize = 10;

        stream::iter(friend_list)
            .for_each_concurrent(CONCURRENT_REQUESTS, |friend| {
                let client = client.clone();
                let pool = pool.clone();
                let our_username = our_username.to_string();
                let our_address = our_address.to_string();

                async move {
                    match process_friend_messages(
                        &pool,
                        &client,
                        &our_username,
                        &our_address,
                        &friend,
                    )
                    .await
                    {
                        Ok(_) => println!("Processed messages from {}", friend.username),
                        Err(e) => {
                            eprintln!("Error processing messages from {}: {}", friend.username, e)
                        }
                    }
                }
            })
            .await;

        //println!("Fetch cycle complete. Waiting for {}s.", sleep_time);
        tokio::time::sleep(Duration::from_secs(sleep_time)).await;
    }
}

pub async fn send_friend_request(
    pool: &SqlitePool,
    client: &Client,
    our_username: &str,
    friend: &Friend,
) -> Result<(), String> {
    let req_body = FriendInput {
        username: friend.username.clone(),
        hostname: our_username.to_string(),
        req_type: friend.status.status_enum(),
    };

    let target_url = format!("http://{}/friend_request", friend.address);

    let response = client
        .post(&target_url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        update_friend_status_as_sent(pool, &friend.username)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        Ok(())
    } else {
        Err(format!("Non-200 response: {}", response.status()))
    }
}

pub async fn friend_fetcher(pool: &SqlitePool, sleep_time: u64) {
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
                    match send_friend_request(&pool, &client, &our_username, &friend).await {
                        Ok(_) => println!("Friend request sent to {}", friend.username),
                        Err(e) => eprintln!("Error sending request to {}: {}", friend.username, e),
                    }
                }
            })
            .await;

        //println!("Friend update cycle complete. Waiting for {}s.", sleep_time);
        tokio::time::sleep(Duration::from_secs(sleep_time)).await;
    }
}
