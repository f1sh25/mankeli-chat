use super::*;
use rusqlite::Connection;

#[test]
fn test_fetch_users_with_data() {
    let conn = Connection::open_in_memory().unwrap();

    let chat_user = User {
        id: 0,
        username: "testuser".to_string(),
        address: "127.0.0.1".to_string(),
    };

    let _ = setup_db(&conn, &chat_user, true).unwrap();

    let result: Result<Vec<Friend>, rusqlite::Error> = fetch_users(&conn);
    let users = result.unwrap();
    assert_eq!(users.len(), 3);
    assert_eq!(users[0].username, "user1");
    assert_eq!(users[0].address, "1.1.1.1");
}

#[test]
fn test_fetch_outgoing_with_data() {
    let conn = Connection::open_in_memory().unwrap();

    let chat_user = User {
        id: 0,
        username: "testuser".to_string(),
        address: "127.0.0.1".to_string(),
    };

    let _ = setup_db(&conn, &chat_user, true).unwrap();

    conn.execute_batch(
        "
        INSERT INTO outgoing (recipient, recipient_address, subject, message, sent) VALUES
            ('charlie@example.com', '1.2.3.4', 'Test subject','Test message', 0);
        ",
    )
    .unwrap();

    let result: Result<Vec<Outgoing>, rusqlite::Error> = fetch_outgoing(&conn);
    assert!(result.is_ok(), "Error fetching outgoing: {:?}", result);
    let outgoing = result.unwrap();
    assert_eq!(outgoing.len(), 3);
    assert_eq!(outgoing[2].recipient, "charlie@example.com");
    assert_eq!(outgoing[2].body, "Test message");
    assert_eq!(outgoing[2].sent, false);
}

#[test]
fn test_send_message() {
    let conn = Connection::open_in_memory().unwrap();

    let chat_user = User {
        id: 0,
        username: "testuser".to_string(),
        address: "127.0.0.1".to_string(),
    };

    let _ = setup_db(&conn, &chat_user, true).unwrap();

    let message = Message {
        send_to: "user3".to_string(),
        subject: "test message".to_string(),
        content: "Hello world!".to_string(),
    };

    let result = send_message_to_que(&conn, &message);
    assert!(result.is_ok());

    // Verify it was inserted
    let mut stmt = conn
        .prepare("SELECT recipient, message FROM outgoing  WHERE recipient = ?1")
        .unwrap();
    let mut rows: rusqlite::Rows<'_> = stmt.query([message.send_to.clone()]).unwrap();
    let row = rows.next().unwrap().unwrap();

    let recipient: String = row.get(0).unwrap();
    let content: String = row.get(1).unwrap();

    assert_eq!(recipient, message.send_to);
    assert_eq!(content, message.content);
}

#[test]
fn test_send_invite() {
    let conn = Connection::open_in_memory().unwrap();

    let chat_user = User {
        id: 0,
        username: "testuser".to_string(),
        address: "127.0.0.1".to_string(),
    };

    let _ = setup_db(&conn, &chat_user, true).unwrap();

    let request = FriendRequest {
        username: "alice".to_string(),
        address: "alice@example.com".to_string(),
    };

    let result = send_invite(&conn, &request);
    assert!(result.is_ok());

    // Verify it was inserted
    let mut stmt = conn
        .prepare("SELECT username, address FROM friends WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([request.username]).unwrap();
    let row = rows.next().unwrap().unwrap();

    let username: String = row.get(0).unwrap();
    let address: String = row.get(1).unwrap();

    assert_eq!(username, "alice");
    assert_eq!(address, "alice@example.com");
}
