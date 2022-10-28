use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

use crate::AppError;

pub async fn regex_limit() -> Result<impl IntoResponse, AppError> {
    return Ok(Json(json!({ "limit": crate::consts::MAX_REGEX_COUNT })));
}