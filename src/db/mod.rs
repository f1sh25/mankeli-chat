use crate::api::FriendRequestStatus;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::{string, sync::mpsc::Sender};

#[cfg(test)]
mod tests;

#[derive(FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub address: String,
}

#[derive(Debug, FromRow)]
pub struct Friend {
    pub id: i64,
    pub username: String,
    pub address: String,
    pub status: i64,
    pub added_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow)]
pub struct InboxMessage {
    pub id: i64,
    pub sender: String,
    pub subject: String,
    pub message: String,
    pub received_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow)]
pub struct Outgoing {
    pub id: i64,
    pub sender: String,
    pub recipient: String,
    pub recipient_address: String,
    pub subject: String,
    pub body: String,
    pub queued_at: Option<NaiveDateTime>,
    pub sent: Option<bool>,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Message {
    pub send_to: String,
    pub subject: String,
    pub content: String,
}

pub struct FriendRequest {
    pub username: String,
    pub address: String,
}

pub async fn setup_db(pool: &SqlitePool, initial_user: &User) -> Result<(), sqlx::Error> {
    let user_exists: Option<i64> = sqlx::query_scalar!(
        "SELECT id FROM user WHERE username = ?",
        initial_user.username
    )
    .fetch_optional(pool)
    .await?;

    if user_exists.is_none() {
        sqlx::query!(
            "INSERT INTO user (username, address) VALUES (?, ?)",
            initial_user.username,
            initial_user.address
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn retr_user(pool: &SqlitePool) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(User, "SELECT id, username, address FROM user LIMIT 1")
        .fetch_one(pool)
        .await?;
    Ok(user)
}

pub async fn fetch_users(pool: &SqlitePool) -> Result<Vec<Friend>, sqlx::Error> {
    let friends = sqlx::query_as!(
        Friend,
        "SELECT id, username, address, status, added_at FROM friends"
    )
    .fetch_all(pool)
    .await?;
    Ok(friends)
}

pub async fn fetch_inbox(pool: &SqlitePool) -> Result<Vec<InboxMessage>, sqlx::Error> {
    let messages = sqlx::query_as!(
        InboxMessage,
        "SELECT id, sender, subject, message, received_at FROM inbox"
    )
    .fetch_all(pool)
    .await?;
    Ok(messages)
}

pub async fn fetch_outgoing(pool: &SqlitePool) -> Result<Vec<Outgoing>, sqlx::Error> {
    let messages = sqlx::query_as!(
        Outgoing,
        "SELECT id, sender, recipient, recipient_address, subject, message as body, queued_at, sent FROM outgoing"
    )
    .fetch_all(pool)
    .await?;
    Ok(messages)
}

pub async fn send_message_to_que(pool: &SqlitePool, message: &Message) -> Result<(), sqlx::Error> {
    let sender = retr_user(pool).await?;

    let recipient_address: (String,) =
        sqlx::query_as("SELECT address FROM friends WHERE username = ?")
            .bind(&message.send_to)
            .fetch_one(pool)
            .await?;

    sqlx::query!(
        "INSERT INTO outgoing (sender, recipient, recipient_address, subject, message) VALUES (?, ?, ?, ?, ?)",
        sender.username,
        message.send_to,
        recipient_address.0,
        message.subject,
        message.content
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn send_invite(pool: &SqlitePool, request: &FriendRequest) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO Friends (username, address) VALUES (?, ?)",
        request.username,
        request.address
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_message(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM inbox WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_user(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM Friends WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn fetch_messages_for_user(
    pool: &SqlitePool,
    username: String,
) -> Result<Vec<Outgoing>, sqlx::Error> {
    let messages: Vec<Outgoing> = sqlx::query_as!(
        Outgoing,
        r#"
        SELECT id, sender, recipient, recipient_address, subject, message as body, queued_at, sent
        FROM outgoing
        WHERE recipient = ?
        "#,
        username
    )
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

//Fetch accepted friends
pub async fn fetch_active_friends(pool: &SqlitePool) -> Result<Vec<Friend>, sqlx::Error> {
    let status = 2;
    let friends: Vec<Friend> = sqlx::query_as!(
        Friend,
        r#"
        SELECT id, username, address, status, added_at
        FROM friends
        WHERE status = ?
        "#,
        status
    )
    .fetch_all(pool)
    .await?;

    Ok(friends)
}

//Fetch unsent requests

pub async fn fetch_unsent_friend_updt(
    pool: &SqlitePool,
) -> Result<(String, Vec<Friend>), sqlx::Error> {
    let user = retr_user(pool).await?;

    let status = false;
    let friends: Vec<Friend> = sqlx::query_as!(
        Friend,
        r#"
        SELECT id, username, address, status, added_at
        FROM friends
        WHERE sent = ?
        "#,
        status
    )
    .fetch_all(pool)
    .await?;

    Ok((user.username, friends))
}

pub async fn batch_ingest(pool: &SqlitePool, messages: Vec<Message>) -> Result<(), sqlx::Error> {
    todo!()
}

pub async fn update_friend_status_as_sent(
    pool: &SqlitePool,
    username: &String,
) -> Result<(), sqlx::Error> {
    todo!()
}
