# Mankeli-chat

Mankeli-chat is a peer-to-peer (P2P) terminal-based chat client written in Rust. It stores all messages locally using SQLite, enabling direct communication between users without relying on a centralized messaging server.

This project was built to learn Rust through a larger and more complete application, exploring async I/O, database interactions, networking, and API design.

## Features
- multiple users per client
- Add and remove friends
- send and receive messages (queued if offline)
- Local message storage with sqlite
- Simple JSON-based configuration

## Tech Stack
- sqlx (Async, compile-time checked SQL)
- axum (HTTP server (used for message communication))
- tokio (Async runtime)
- sqlite (Local Persistent storage)

## Configuration
### Example config.json file
```
{
  "server_address": "127.0.0.1:3000",
  "message_fetch_interval": 5,
  "friend_fetch_interval": 10
}
```
- ```Server_address```: Local address to bind the Axum server
- ```message_fetch_interval```: Interval (in seconds) to fetch messages

- ```friend_fetch_interval```: Interval (in seconds) to refresh friend list



## Getting started

Rust is required and it can be installed from: https://rustup.rs
### Installation

```
git clone https://github.com/f1sh25/mankeli-chat
cd mankeli-chat
cargo run
```

## Usage
**Once started**
- Enter username (new or existing)

### Commands available

```
inbox      - View received messages
friends    - View/add/remove friends or handle invites
send       - Send a message to a friend
outbound   - View sent messages
quit       - Exit the application
```

## Lessons learned / Challenges

- Designing async applications using tokio and axum
- Writing modular and testable Rust functions
- Structuring data models and async workflows
- Implementing basic peer-to-peer communication
- Rust error handling and mapping
- Managing concurrency with multiple background tasks

## Improvements
- Add verbose/debug logging for diagnostics
- Improve error propagation and helpful messages
- Add HTTPS support for secure connections
- End-to-end encryption using PGP keys
    - Automatically exchange keys when accepting a friend request
- Better message delivery guarantees (e.g., retries, receipts)
- Better terminal UI