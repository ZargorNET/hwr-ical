use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use axum::body::Body;
use axum::extract::Query;
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Router;
use encoding_rs::UTF_8;

const REVERSE_PROXY_URL: &str = "https://moodle.hwr-berlin.de/fb2-stundenplan/download.php?doctype=.ics&url=";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();

    let app = Router::new()
        .fallback(root.into_service());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let target_url = match params.get("url") {
        Some(x) => x,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("url query is missing")).unwrap();
        }
    };


    let res = reqwest::get(REVERSE_PROXY_URL.to_owned() + target_url).await.unwrap();
    let body = res.bytes().await.unwrap();
    let res = UTF_8.decode_with_bom_removal(&body).0.to_string();

    Response::builder()
        .header("Content-Disposition", "inline; filename=calendar.ics")
        .header("Content-Type", "text/calendar; charset=utf-8")
        .status(StatusCode::OK)
        .body(Body::from(res)).unwrap()
}
