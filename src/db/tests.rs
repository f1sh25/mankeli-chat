use super::*;
use rusqlite::Connection;

#[test]
fn test_fetch_inbox_with_data() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "
        CREATE TABLE inbox (
            id INTEGER PRIMARY KEY, 
            sender TEXT NOT NULL,
            message TEXT NOT NULL,
            received_at TEXT NOT NULL
        );
        INSERT INTO inbox (sender, message, received_at) VALUES
            ('alice@example.com', 'Hello, world!', '2025-07-10 12:00:00');
        ",
    )
    .unwrap();

    let result: Result<Vec<InboxMessage>, rusqlite::Error> = fetch_inbox(&conn);
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].sender, "alice@example.com");
    assert_eq!(messages[0].message, "Hello, world!");
}

#[test]
fn test_fetch_users_with_data() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "
        CREATE TABLE friends (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            address TEXT NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO friends (username, address) VALUES
            ('bob', 'bob@domain.com');
        ",
    )
    .unwrap();

    let result: Result<Vec<Friend>, rusqlite::Error> = fetch_users(&conn);
    assert!(result.is_ok());
    let users = result.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].username, "bob");
    assert_eq!(users[0].address, "bob@domain.com");
}

#[test]
fn test_fetch_outgoing_with_data() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "
        CREATE TABLE outgoing (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient TEXT NOT NULL,
            message TEXT NOT NULL,
            queued_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            sent BOOLEAN DEFAULT 0
        );
        INSERT INTO outgoing (recipient, message, sent) VALUES
            ('charlie@example.com', 'Test message', 0);
        ",
    )
    .unwrap();

    let result: Result<Vec<Outgoing>, rusqlite::Error> = fetch_outgoing(&conn);
    assert!(result.is_ok(), "Error fetching outgoing: {:?}", result);
    let outgoing = result.unwrap();
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].recipient, "charlie@example.com");
    assert_eq!(outgoing[0].body, "Test message");
    assert_eq!(outgoing[0].sent, false);
}

#[test]
fn test_send_message() {
    let conn = Connection::open_in_memory().unwrap();

    conn.execute_batch(
        "CREATE TABLE outgoing (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient TEXT NOT NULL,
            message TEXT NOT NULL,
            queued_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            sent BOOLEAN DEFAULT 0
        );",
    )
    .unwrap();

    let message = Message {
        send_to: "user@example.com".to_string(),
        content: "Hello world!".to_string(),
        // other fields if needed
    };

    let result = send_message(&conn, message);
    assert!(result.is_ok());

    // Verify it was inserted
    let mut stmt = conn
        .prepare("SELECT recipient, message FROM outgoing")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();

    let recipient: String = row.get(0).unwrap();
    let content: String = row.get(1).unwrap();

    assert_eq!(recipient, "user@example.com");
    assert_eq!(content, "Hello world!");
}

#[test]
fn test_send_invite() {
    let conn = Connection::open_in_memory().unwrap();

    conn.execute_batch(
        "CREATE TABLE friends (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            address TEXT NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
    )
    .unwrap();

    let request = FriendRequest {
        username: "alice".to_string(),
        address: "alice@example.com".to_string(),
    };

    let result = send_invite(&conn, request);
    assert!(result.is_ok());

    // Verify it was inserted
    let mut stmt = conn
        .prepare("SELECT username, address FROM friends")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();

    let username: String = row.get(0).unwrap();
    let address: String = row.get(1).unwrap();

    assert_eq!(username, "alice");
    assert_eq!(address, "alice@example.com");
}
