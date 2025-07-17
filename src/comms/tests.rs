use super::*;
use crate::api::FetchMessageResponse;
use httpmock::{Method::POST, MockServer};
use reqwest::Client;
use sqlx::{SqlitePool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_process_friend_messages_success() {
    let server = MockServer::start();

    let friend = Friend {
        id: 1,
        username: "alice".into(),
        address: server.address().to_string(),
        status: 2,
        added_at: None,
    };
    let _mock = server.mock(|when, then| {
        when.method(POST).path("/fetch_messages");
        then.status(200).json_body_obj(&FetchMessageResponse {
            messages: vec![Message {
                sender: "alice".into(),
                subject: "hi".into(),
                body: "hello".into(),
            }],
        });
    });

    let client = Client::new();
    let pool = setup_test_db().await;

    let result = process_friend_messages(&pool, &client, "bob", "1.2.3.4", &friend)
        .await
        .map_err(|e| eprintln!("{}", e));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_friend_request_success() {
    let server = MockServer::start();

    let friend = Friend {
        id: 2,
        username: "carol".into(),
        address: server.address().to_string(),
        status: 1,
        added_at: None,
    };

    let _mock = server.mock(|when, then| {
        when.method(POST).path("/friend_request");
        then.status(200);
    });

    let client = Client::new();
    let pool = setup_test_db().await;

    let result = send_friend_request(&pool, &client, "bob", &friend).await;

    assert!(result.is_ok());
}
