use super::*;
use crate::db::{MIGRATOR, OutgoingMessage, User, send_message_to_que, setup_db};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::post,
};
use serde_json::json;
use sqlx::SqlitePool;
use std::usize;
use tokio;
use tower::ServiceExt;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    pool
}

async fn send_test_messages(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO friends (username, address) VALUES ('user3', '3.3.3.3')")
        .execute(pool)
        .await?;

    let message = OutgoingMessage {
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

#[tokio::test]
async fn test_friend_invite_sent_and_accepted_successfully() {
    let pool = setup_test_db().await;
    let shared_pool = Arc::new(pool.clone());

    let app = Router::new()
        .route("/friend_request", post(super::friend_request_handler))
        .layer(Extension(shared_pool));
    // send invite
    let invite_input = FriendInput {
        username: "alice".into(),
        hostname: "bob".into(),
        address: "1.1.1.1".into(),
        req_type: FriendRequestStatus::InviteSent,
    };

    let invite_body = serde_json::to_string(&invite_input).unwrap();

    let invite_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/friend_request")
                .header("Content-Type", "application/json")
                .body(Body::from(invite_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invite_response.status(), StatusCode::OK);

    let invite_body_bytes = to_bytes(invite_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read invite response body");
    let invite_json: serde_json::Value = serde_json::from_slice(&invite_body_bytes).unwrap();

    assert_eq!(invite_json["status"], "invite_sent");

    let row: (i64,) = sqlx::query_as("SELECT status FROM friends WHERE username = 'alice'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 1);

    // Step 2: Accept invite
    let accept_input = FriendInput {
        username: "alice".into(),
        hostname: "bob".into(),
        address: "1.1.1.1".into(),
        req_type: FriendRequestStatus::Accepted,
    };

    let accept_body = serde_json::to_string(&accept_input).unwrap();

    let accept_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/friend_request")
                .header("Content-Type", "application/json")
                .body(Body::from(accept_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(accept_response.status(), StatusCode::OK);

    let accept_body_bytes = to_bytes(accept_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read accept response body");
    let accept_json: serde_json::Value = serde_json::from_slice(&accept_body_bytes).unwrap();

    assert_eq!(accept_json["status"], "accepted");

    let updated_row: (i64,) = sqlx::query_as("SELECT status FROM friends WHERE username = 'alice'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(updated_row.0, 2); // 2 means Accepted
}
