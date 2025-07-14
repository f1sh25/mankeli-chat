use rusqlite::{Connection, Result, params};
use std::{path, vec};
#[cfg(test)]
mod tests;
pub struct User {
    pub id: i16,
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
            message TEXT NOT NULL,
            received_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS outgoing (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient TEXT NOT NULL,
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
#[derive(Debug)]
pub struct Friend {
    id: i32,
    username: String,
    address: String,
    added_at: String,
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

#[derive(Debug)]
pub struct InboxMessage {
    id: i32,
    sender: String,
    message: String,
    received_at: String,
}

pub fn fetch_inbox(conn: &Connection) -> Result<Vec<InboxMessage>> {
    let mut stmt = conn.prepare("SELECT id, sender, message, received_at FROM inbox")?;
    let inbox_iter = stmt.query_map([], |row| {
        Ok(InboxMessage {
            id: row.get(0)?,
            sender: row.get(1)?,
            message: row.get(2)?,
            received_at: row.get(3)?,
        })
    })?;

    let mut result = Vec::new();

    for message in inbox_iter {
        result.push(message?);
    }

    Ok(result)
}

#[derive(Debug)]
pub struct Outgoing {
    id: i16,
    recipient: String,
    body: String,
    queued_at: String,
    sent: bool,
}

pub fn fetch_outgoing(conn: &Connection) -> Result<Vec<Outgoing>> {
    let mut stmt = conn.prepare("SELECT id, recipient, message, queued_at, sent FROM outgoing")?;
    let outgoing_iter = stmt.query_map([], |row| {
        Ok(Outgoing {
            id: row.get(0)?,
            recipient: row.get(1)?,
            body: row.get(2)?,
            queued_at: row.get(3)?,
            sent: row.get(4)?,
        })
    })?;

    let mut result = Vec::new();

    for message in outgoing_iter {
        result.push(message?);
    }

    Ok(result)
}

pub struct Message {
    send_to: String,
    content: String,
}

pub fn send_message(conn: &Connection, message: Message) -> Result<()> {
    conn.execute(
        "INSERT INTO outgoing (recipient, message) VALUES (?1, ?2)",
        params![message.send_to, message.content],
    )?;
    Ok(())
}

pub struct FriendRequest {
    username: String,
    address: String,
}

pub fn send_invite(conn: &Connection, request: FriendRequest) -> Result<()> {
    conn.execute(
        "INSERT INTO Friends (username, address) VALUES (?1, ?2)",
        params![request.username, request.address],
    )?;

    Ok(())
}
