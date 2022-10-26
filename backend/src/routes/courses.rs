use axum::{Extension, Json};
use axum::response::IntoResponse;
use serde_json::json;

use crate::{AppError, AppState};

pub async fn courses(Extension(state): Extension<AppState>) -> Result<impl IntoResponse, AppError> {
    let readGuard = state.course_fetcher.course.read().await;
    Ok(Json(json!({"courses": readGuard.deref()})))
}