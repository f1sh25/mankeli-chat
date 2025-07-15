use super::*;
use crate::db::{FriendRequest, Message, User, send_invite, send_message_to_que, setup_db};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response,
    routing::post,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::usize;
use tokio;
use tower::ServiceExt;
fn app() -> Router {
    Router::new().route("/fetch_messages", post(fetch_messages_handler))
}

fn send_test_messages(conn: &Connection) -> Result<(), rusqlite::Error> {
    let user1 = FriendRequest {
        username: "user1".to_string(),
        address: "1.1.1.1".to_string(),
    };

    let _ = send_invite(conn, &user1)?;

    let user2 = FriendRequest {
        username: "user2".to_string(),
        address: "2.2.2.2".to_string(),
    };

    let _ = send_invite(conn, &user2)?;

    let message1 = Message {
        send_to: "user1".to_string(),
        subject: "test subject".to_string(),
        content: "test content".to_string(),
    };

    let _ = send_message_to_que(conn, &message1)?;

    let message2 = Message {
        send_to: "user2".to_string(),
        subject: "test subject".to_string(),
        content: "test content".to_string(),
    };

    let _ = send_message_to_que(conn, &message2)?;

    Ok(())
}

#[tokio::test]
async fn test_fetch_messages_success() {
    let app = app();

    let conn = Connection::open_in_memory().unwrap();

    let chat_user = User {
        id: 0,
        username: "testuser".to_string(),
        address: "127.0.0.1".to_string(),
    };

    let _ = setup_db(&conn, &chat_user, true).unwrap();

    let input = FetchMessageInput {
        username: "user1".to_string(),
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
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: FetchMessageResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(response_json.messages.len(), 1);

    let message = &response_json.messages[0];
    assert_eq!(message.sender, "testuser");
    assert_eq!(message.subject, "test subject");
    assert_eq!(message.body, "test content");
}
