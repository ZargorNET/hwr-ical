use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Extension, Json, Router};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde_json::json;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::course_fetcher::CourseFetcher;

mod routes;
mod ical_parser;
mod consts;
mod course_fetcher;

pub struct AppError(anyhow::Error);

#[derive(Clone)]
pub struct AppState {
    pub course_fetcher: Arc<CourseFetcher>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();


    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();

    let app_state = AppState {
        course_fetcher: Arc::new(CourseFetcher {
            course: Default::default()
        })
    };

    course_fetcher::start(app_state.course_fetcher.clone());


    let app = Router::new()
        .route("/regex_limit", any(routes::regex_limit))
        .route("/courses", any(routes::courses))
        .route("/:study/:semester/:course/*regex", any(routes::format_ical))
        .fallback(any(not_found))
        .layer(Extension(app_state));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": &self.0.to_string()}))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}
