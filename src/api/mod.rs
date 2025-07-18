use std::sync::Arc;

use crate::db::fetch_messages_for_user;
use axum::{
    Extension, Router, extract::Json, http::StatusCode, response::IntoResponse, routing::get,
    routing::post,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::error;

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
    pub address: String,
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

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "Hello, this is a mankeli-chat server" }),
        )
        .route("/fetch_messages", post(fetch_messages_handler))
        .route("/friend_request", post(friend_request_handler))
        .layer(Extension(Arc::new(pool)))
}

pub async fn mark_messages_as_sent(
    pool: &SqlitePool,
    message_ids: &[i64],
) -> Result<(), sqlx::Error> {
    if message_ids.is_empty() {
        return Ok(());
    }

    let placeholders = std::iter::repeat("?")
        .take(message_ids.len())
        .collect::<Vec<_>>()
        .join(",");

    let sql = format!(
        "UPDATE outgoing SET sent = 1 WHERE id IN ({})",
        placeholders
    );

    let mut query = sqlx::query(&sql);
    for id in message_ids {
        query = query.bind(id);
    }

    query.execute(pool).await?;

    Ok(())
}

pub async fn fetch_messages_handler(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(input): Json<FetchMessageInput>,
) -> Result<Json<FetchMessageResponse>, ApiError> {
    let username = input.username;

    let db_messages = fetch_messages_for_user(&pool, username)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let message_ids: Vec<i64> = db_messages.iter().map(|msg| msg.id).collect();

    mark_messages_as_sent(&pool, &message_ids)
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
    Json(input): Json<FriendInput>,
) -> impl IntoResponse {
    let FriendInput {
        username,
        hostname,
        address,
        req_type,
    } = input;

    match req_type {
        FriendRequestStatus::InviteSent => {
            // Insert invite_sent from A to B
            let res = sqlx::query!(
                r#"
                INSERT INTO friends (username, address, status, sent)
                VALUES (?, ?, 1, 1)
                ON CONFLICT(username) DO UPDATE SET
                    status = 1,
                    added_at = CURRENT_TIMESTAMP
                "#,
                hostname,
                address,
            )
            .execute(&*pool)
            .await;

            match res {
                Ok(_) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "status": "invite_sent" })),
                )
                    .into_response(),
                Err(e) => {
                    error!("Failed to send invite: {:?}", e);
                    ApiError::InternalServerError("DB insert failed".into()).into_response()
                }
            }
        }
        FriendRequestStatus::InviteReceived => {
            return ApiError::InvalidInput("why would you request this".to_string())
                .into_response();
        }
        FriendRequestStatus::Accepted => {
            let existing: Option<i64> = match sqlx::query_scalar!(
                r#"
                SELECT status as "status: i64" FROM friends
                WHERE username = ? AND address = ?
                "#,
                hostname,
                address
            )
            .fetch_optional(&*pool)
            .await
            {
                Ok(val) => val,
                Err(e) => {
                    error!("DB check failed: {:?}", e);
                    return ApiError::InternalServerError("DB check failed".into()).into_response();
                }
            };

            if let Some(s) = existing {
                if s == 0 {
                    let res = sqlx::query!(
                        r#"
                        INSERT INTO friends (username, address, status, sent)
                        VALUES (?, ?, 2, 1)
                        ON CONFLICT(username) DO UPDATE SET
                            status = 2,
                            added_at = CURRENT_TIMESTAMP
                        "#,
                        hostname,
                        address
                    )
                    .execute(&*pool)
                    .await;

                    match res {
                        Ok(_) => (
                            StatusCode::OK,
                            Json(serde_json::json!({ "status": "accepted" })),
                        )
                            .into_response(),
                        Err(e) => {
                            error!("Failed to accept friend: {:?}", e);
                            ApiError::InternalServerError("Failed to accept friend".into())
                                .into_response()
                        }
                    }
                } else {
                    ApiError::InvalidInput("No pending invitation to accept.".into())
                        .into_response()
                }
            } else {
                ApiError::NotFound("No invitation found.".into()).into_response()
            }
        }
        FriendRequestStatus::Rejected => {
            let updated = sqlx::query!(
                r#"
                UPDATE friends SET status = 3, sent=1 ,added_at = CURRENT_TIMESTAMP
                WHERE username = ? AND address = ? AND status = 0
                "#,
                hostname,
                address
            )
            .execute(&*pool)
            .await;

            match updated {
                Ok(result) => {
                    if result.rows_affected() == 0 {
                        ApiError::InvalidInput("No pending invitation to reject.".into())
                            .into_response()
                    } else {
                        (
                            StatusCode::OK,
                            Json(serde_json::json!({ "status": "rejected" })),
                        )
                            .into_response()
                    }
                }
                Err(e) => {
                    error!("Failed to reject friend: {:?}", e);
                    ApiError::InternalServerError("Failed to reject friend.".into()).into_response()
                }
            }
        }
    }
}
