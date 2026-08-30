use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::AppError;

pub async fn regex_limit() -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({ "limit": crate::consts::MAX_REGEX_COUNT })))
}

