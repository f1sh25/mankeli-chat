use axum::{Router, extract::Json, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
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

pub async fn fetch_messages_handler(
    Json(input): Json<FetchMessageInput>,
) -> Result<Json<FetchMessageResponse>, ApiError> {
    todo!()
}
