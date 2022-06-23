use std::error::Error;
use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Router;
use axum::routing::any;
use encoding_rs::UTF_8;
use regex::{Regex, RegexBuilder};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use ical_parser::ParsedCalendar;

use crate::ical_parser::ParsedEvent;

mod ical_parser;

const REVERSE_PROXY_URL: &str = "https://moodle.hwr-berlin.de/fb2-stundenplan/download.php?doctype=.ics&url=./fb2-stundenplaene/";

//informatik/semester3/kursa
struct AppError(anyhow::Error);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();

    let app = Router::new()
        .route("/:study/:semester/:course/*regex", any(root));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root(Path((study, semester, course, rregex)): Path<(String, String, String, String)>) -> Result<Response<Body>, AppError> {
    let regex = &rregex[1..];

    let res = reqwest::get(REVERSE_PROXY_URL.to_owned() + &format!("{study}/{semester}/{course}")).await.unwrap();
    let body = res.bytes().await.unwrap();
    let mut res = UTF_8.decode_with_bom_removal(&body).0.to_string();

    if !regex.is_empty() {
        let built_regex: Vec<Regex> = regex.split("/")
            .filter_map(|r| RegexBuilder::new(&r).build().ok())
            .collect();
        res = res.replace("\r", ""); // why? But okayyy
        let cal = ParsedCalendar::from(res.as_str());

        let mut events_to_remove: Vec<&ParsedEvent> =
            cal.events.iter()
                .filter(|e| built_regex.iter().any(|r| r.is_match(e.summary)))
                .collect();

        events_to_remove.sort_by(|a, b| a.start.cmp(&b.start));

        let mut copied_res = res.to_owned();
        let mut to_subtract = 0;

        for event in events_to_remove {
            let start = event.start - to_subtract;
            let end = event.end - to_subtract;
            copied_res.replace_range(start..end, "");
            to_subtract += end - start;
        }
        res = copied_res;
    }

    Ok(
        Response::builder()
            .header("Content-Disposition", "inline; filename=calendar.ics")
            .header("Content-Type", "text/calendar; charset=utf-8")
            .status(StatusCode::OK)
            .body(Body::from(res)).unwrap()
    )
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "app error").into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}