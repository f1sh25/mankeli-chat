use rusqlite::{Connection, Result, params};
use std::{path, vec};
#[cfg(test)]
mod tests;
pub struct User {
    pub id: i16,
    pub username: String,
    pub address: String,
}

#[derive(Debug)]
pub struct Friend {
    pub id: i32,
    pub username: String,
    pub address: String,
    pub added_at: String,
}

#[derive(Debug)]
pub struct InboxMessage {
    pub id: i32,
    pub sender: String,
    pub subject: String,
    pub message: String,
    pub received_at: String,
}

#[derive(Debug)]
pub struct Outgoing {
    pub id: i16,
    pub recipient: String,
    pub recipient_address: String,
    pub subject: String,
    pub body: String,
    pub queued_at: String,
    pub sent: bool,
}

pub struct Message {
    pub send_to: String,
    pub subject: String,
    pub content: String,
}

pub struct FriendRequest {
    pub username: String,
    pub address: String,
}

pub fn retr_user(conn: &Connection) -> Result<User> {
    conn.query_row(
        "SELECT id, username, address FROM user LIMIT 1",
        params![],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                address: row.get(2)?,
            })
        },
    )
}

pub fn setup_db(conn: &Connection, user: &User) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS inbox (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender TEXT NOT NULL,
            subject TEXT NOT NULL,
            message TEXT NOT NULL,
            received_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS outgoing (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient TEXT NOT NULL,
            recipient_address TEXT NOT NULL,
            subject TEXT NOT NULL,
            message TEXT NOT NULL,
            queued_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            sent BOOLEAN DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS friends (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            address TEXT NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            status TEXT NOT NULL DEFAULT 'requested'
                CHECK (status IN ('requested', 'accepted', 'rejected'))
        );


        CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            address TEXT NOT NULL
        );

        
        ",
    )?;
    conn.execute(
        "INSERT INTO user (username, address) VALUES (?1, ?2)",
        params![user.username, user.id],
    )?;

    Ok(())
}

pub fn fetch_users(conn: &Connection) -> Result<Vec<Friend>> {
    // with path test can run in memory db
    //let conn = Connection::open(path)?;

    let mut stmt = conn.prepare("SELECT id, username, address, added_at FROM friends")?;
    let friends_iter = stmt.query_map([], |row| {
        Ok(Friend {
            id: row.get(0)?,
            username: row.get(1)?,
            address: row.get(2)?,
            added_at: row.get(3)?,
        })
    })?;

    let mut result = Vec::new();

    for friend in friends_iter {
        result.push(friend?);
    }

    Ok(result)
}

pub fn fetch_inbox(conn: &Connection) -> Result<Vec<InboxMessage>> {
    let mut stmt = conn.prepare("SELECT id, sender, subject, message, received_at FROM inbox")?;
    let inbox_iter = stmt.query_map([], |row| {
        Ok(InboxMessage {
            id: row.get(0)?,
            sender: row.get(1)?,
            subject: row.get(2)?,
            message: row.get(3)?,
            received_at: row.get(4)?,
        })
    })?;

    let mut result = Vec::new();

    for message in inbox_iter {
        result.push(message?);
    }

    Ok(result)
}

pub fn fetch_outgoing(conn: &Connection) -> Result<Vec<Outgoing>> {
    let mut stmt = conn.prepare(
        "SELECT id, recipient, recipient_address, subject, message, queued_at, sent FROM outgoing",
    )?;
    let outgoing_iter = stmt.query_map([], |row| {
        Ok(Outgoing {
            id: row.get(0)?,
            recipient: row.get(1)?,
            recipient_address: row.get(2)?,
            subject: row.get(3)?,
            body: row.get(4)?,
            queued_at: row.get(5)?,
            sent: row.get(6)?,
        })
    })?;

    let mut result = Vec::new();

    for message in outgoing_iter {
        result.push(message?);
    }

    Ok(result)
}

pub fn send_message_to_que(conn: &Connection, message: Message) -> Result<()> {
    let mut stmt = conn.prepare("SELECT address FROM friends WHERE username = ?1")?;
    //to-do make this so it prints that user wasnt found
    let recipient_address: String = stmt.query_row(params![message.send_to], |row| row.get(0))?;

    conn.execute(
        "INSERT INTO outgoing (recipient, recipient_address, subject, message) VALUES (?1, ?2, ?3, ?4)",
        params![message.send_to, recipient_address, message.subject, message.content],
    )?;
    Ok(())
}

pub fn send_invite(conn: &Connection, request: FriendRequest) -> Result<()> {
    conn.execute(
        "INSERT INTO Friends (username, address) VALUES (?1, ?2)",
        params![request.username, request.address],
    )?;

    //add logic here to send friend request to api
    //if fails maybe start another thread that will call it
    Ok(())
}

pub fn delete_message(conn: &Connection, id: i32) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM inbox WHERE id = ?", &[&id])?;
    Ok(())
}

pub fn delete_user(conn: &Connection, id: i32) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM Friends WHERE id = ?", &[&id])?;
    Ok(())
}

pub fn fetch_test_inbox(_conn: &Connection) -> rusqlite::Result<Vec<InboxMessage>> {
    Ok(vec![
        InboxMessage {
            id: 1,
            sender: "alice@example.com".to_string(),
            subject: "Meeting Reminder".to_string(),
            message: "Don't forget our meeting tomorrow at 10am.".to_string(),
            received_at: "6:69".to_string(),
        },
        InboxMessage {
            id: 2,
            sender: "bob@example.com".to_string(),
            subject: "Lunch Plans".to_string(),
            message: "Want to grab lunch this Friday?".to_string(),
            received_at: "6:69".to_string(),
        },
    ])
}
