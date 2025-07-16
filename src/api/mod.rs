use std::sync::Arc;

use crate::db::fetch_messages_for_user;
use axum::{
    Extension, Router, extract::Json, http::StatusCode, response::IntoResponse, routing::post,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, error, info};
#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchMessageInput {
    pub username: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub sender: String,
    pub subject: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchMessageResponse {
    pub messages: Vec<Message>,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum FriendRequestStatus {
    InviteSent,
    InviteReceived,
    Accepted,
    Rejected,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FriendInput {
    pub username: String,
    pub hostname: String,
    pub req_type: FriendRequestStatus,
}

#[derive(Debug, Serialize)]
pub enum ApiError {
    InvalidInput(String),
    NotFound(String),
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (
            status,
            Json(serde_json::json!({
                "error": error_message,
            })),
        )
            .into_response()
    }
}

fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/fetch_messages", post(fetch_messages_handler))
        .layer(Extension(Arc::new(pool)))
}

pub async fn fetch_messages_handler(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(input): Json<FetchMessageInput>,
) -> Result<Json<FetchMessageResponse>, ApiError> {
    let username = input.username;

    let db_messages = fetch_messages_for_user(&pool, username)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let messages: Vec<Message> = db_messages
        .into_iter()
        .map(|msg| Message {
            sender: msg.sender,
            subject: msg.subject,
            body: msg.body,
        })
        .collect();

    let response = FetchMessageResponse { messages };

    Ok(Json(response))
}

pub async fn friend_request_handler(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(input): Json<FetchMessageInput>,
) {
    //post with username and host name with flag Request, Accept, Reject
    todo!()
}
