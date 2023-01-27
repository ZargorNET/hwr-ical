use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::anyhow;

use axum::{Extension, Json, Router};
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::any;
use axum_extra::routing::SpaRouter;
use reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use serde_json::json;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::course_fetcher::CourseFetcher;

mod routes;
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
        .merge(SpaRouter::new("/frontend/", "dist"))
        .route("/:study/:semester/:course/*regex", any(routes::format_ical))
        .route("/regex_limit", any(routes::regex_limit))
        .route("/courses", any(routes::courses))
        .route("/", any(|| async move { Redirect::temporary("/frontend") }))
        .layer(axum::middleware::from_fn(|req: Request<Body>, next: Next<Body>| async {
            let mut cors_headers = HeaderMap::new();
            cors_headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().map_err(|e| anyhow!("could not parse header {}", &e))?);

            // Handle OPTIONS without reading body / etc.
            // While it does not check if the route even exists & co, this will allow CORS
            if req.method() == Method::OPTIONS {
                return Ok::<_, AppError>((StatusCode::OK, cors_headers).into_response());
            }
            let mut res = next.run(req).await;
            cors_headers.into_iter().for_each(|(name, value)| { res.headers_mut().insert(name.unwrap(), value); });

            Ok::<_, AppError>(res)
        }))
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
