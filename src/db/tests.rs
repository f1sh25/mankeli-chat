use super::*;
use sqlx::{SqlitePool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_fetch_users_with_data() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO user (id, username, address) VALUES (0, 'testuser', '127.0.0.1')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO friends (username, address) VALUES ('user1', '1.1.1.1'), ('user2', '2.2.2.2'), ('user3', '3.3.3.3')")
        .execute(&pool)
        .await
        .unwrap();

    let result = fetch_users(&pool).await;
    assert!(result.is_ok());
    let users = result.unwrap();

    assert_eq!(users.len(), 3);
    assert_eq!(users[0].username, "user1");
    assert_eq!(users[0].address, "1.1.1.1");
}

#[tokio::test]
async fn test_fetch_outgoing_with_data() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO user (id, username, address) VALUES (0, 'testuser', '127.0.0.1')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO outgoing (sender, recipient, recipient_address, subject, message, sent) VALUES ('testuser', 'charlie@example.com', '1.2.3.4', 'Test subject','Test message', 0)")
        .execute(&pool)
        .await
        .unwrap();

    let result = fetch_outgoing(&pool).await;
    assert!(result.is_ok(), "Error fetching outgoing: {:?}", result);
    let outgoing = result.unwrap();

    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].recipient, "charlie@example.com");
    assert_eq!(outgoing[0].body, "Test message");
    assert_eq!(outgoing[0].sent, Some(false));
}

#[tokio::test]
async fn test_send_message() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO user (id, username, address) VALUES (0, 'testuser', '127.0.0.1')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO friends (username, address) VALUES ('user3', '3.3.3.3')")
        .execute(&pool)
        .await
        .unwrap();

    let message = OutgoingMessage {
        send_to: "user3".to_string(),
        subject: "test message".to_string(),
        content: "Hello world!".to_string(),
    };

    let result = send_message_to_que(&pool, &message).await;
    assert!(result.is_ok());

    let sent_message = sqlx::query_as::<_, (String, String)>(
        "SELECT recipient, message FROM outgoing WHERE recipient = ?",
    )
    .bind(&message.send_to)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(sent_message.0, message.send_to);
    assert_eq!(sent_message.1, message.content);
}

#[tokio::test]
async fn test_send_invite() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO user (id, username, address) VALUES (0, 'testuser', '127.0.0.1')")
        .execute(&pool)
        .await
        .unwrap();

    let request = FriendRequest {
        username: "alice".to_string(),
        address: "alice@example.com".to_string(),
    };

    let result = send_invite(&pool, &request).await;
    assert!(result.is_ok());

    let friend = sqlx::query_as::<_, (String, String)>(
        "SELECT username, address FROM friends WHERE username = ?",
    )
    .bind(&request.username)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(friend.0, "alice");
    assert_eq!(friend.1, "alice@example.com");
}
