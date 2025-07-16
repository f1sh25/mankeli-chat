use super::*;
use crate::db::{
    FriendRequest, Message, User, fetch_outgoing, fetch_users, send_invite, send_message_to_que,
    setup_db,
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response,
    routing::post,
};
use serde_json::{Value, json};
use sqlx::{SqlitePool, migrate::Migrator};
use std::usize;
use tokio;
use tower::ServiceExt;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    pool
}

async fn send_test_messages(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO friends (username, address) VALUES ('user3', '3.3.3.3')")
        .execute(pool)
        .await?;

    let message = Message {
        send_to: "user3".to_string(),
        subject: "test message".to_string(),
        content: "Hello world!".to_string(),
    };

    let _result = send_message_to_que(&pool, &message).await?;

    Ok(())
}

#[tokio::test]
async fn test_fetch_messages_success() {
    let pool = setup_test_db().await;
    let chat_user = User {
        id: 0,
        username: "testuser".to_string(),
        address: "127.0.0.1".to_string(),
    };
    setup_db(&pool, &chat_user)
        .await
        .expect("Failed to setup initial user");
    send_test_messages(&pool)
        .await
        .expect("Failed to send test messages");

    let app = app(pool);
    let input = FetchMessageInput {
        username: "user3".to_string(),
        address: "1.1.1.1".to_string(),
    };
    let body = json!(input).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/fetch_messages")
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .expect("Failed to get response from app");

    assert_eq!(response.status(), StatusCode::OK);
    let response_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let response_json: FetchMessageResponse =
        serde_json::from_slice(&response_body).expect("Failed to deserialize response JSON");
    assert_eq!(response_json.messages.len(), 1);
    let message = &response_json.messages[0];
    assert_eq!(message.sender, "testuser");
    assert_eq!(message.subject, "test message");
    assert_eq!(message.body, "Hello world!");
}
