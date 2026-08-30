use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde_json::json;

use crate::{AppError, AppState};

pub async fn courses(Extension(state): Extension<AppState>) -> Result<impl IntoResponse, AppError> {
    let read_guard = state.course_fetcher.course.read().await;
    Ok(Json(json!({"courses": *read_guard})))
}

