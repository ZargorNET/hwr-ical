use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Response;
use encoding_rs::UTF_8;
use regex::{Regex, RegexBuilder};

use crate::AppError;
use crate::consts::{MAX_REGEX_COUNT, REVERSE_PROXY_URL};
use crate::ical_parser::{ParsedCalendar, ParsedEvent};

pub async fn root(Path((study, semester, course, rregex)): Path<(String, String, String, String)>) -> Result<Response<Body>, AppError> {
    let regex = &rregex[1..];

    let res = reqwest::get(REVERSE_PROXY_URL.to_owned() + &format!("{study}/{semester}/{course}")).await.unwrap();
    let status = res.status();
    let body = res.bytes().await.unwrap();
    let mut res = UTF_8.decode_with_bom_removal(&body).0.to_string();
    if !status.is_success() || res.contains(r#"alert("Fehlerhafter Pfad!");"#) {
        return Err(anyhow!("proxy error. is your path correct?").into());
    }

    if !regex.is_empty() {
        res = reduce(regex, &res)?;
    }

    Ok(
        Response::builder()
            .header("Content-Disposition", "inline; filename=calendar.ics")
            .header("Content-Type", "text/calendar; charset=utf-8")
            .status(StatusCode::OK)
            .body(Body::from(res)).unwrap()
    )
}

fn reduce(regex: &str, res: &str) -> anyhow::Result<String> {
    let mut return_string;
    let regex_split: Vec<&str> = regex.split("/").collect();

    if regex_split.len() > MAX_REGEX_COUNT as usize {
        return Err(anyhow!("too many regex").into());
    }

    let built_regex: Vec<Regex> = regex_split
        .iter()
        .filter_map(|r| RegexBuilder::new(&r).size_limit(100000).build().ok())
        .collect();
    return_string = res.replace("\r", ""); // why? But okayyy
    let cal = ParsedCalendar::from(return_string.as_str());

    let mut events_to_remove: Vec<&ParsedEvent> =
        cal.events.iter()
            .filter(|e| built_regex.iter().any(|r| r.is_match(&e.summary)))
            .collect();

    events_to_remove.sort_by(|a, b| a.start.cmp(&b.start));

    let mut copied_res = return_string.to_owned();
    let mut to_subtract = 0;

    for event in events_to_remove {
        let start = event.start - to_subtract;
        let end = event.end - to_subtract;
        copied_res.replace_range(start..end, "");
        to_subtract += end - start;
    }
    return_string = copied_res;

    Ok(return_string)
}


#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_CALENDAR: &'static str = include_str!("../../EXAMPLE_CALENDAR.ics");

    #[test]
    fn test_reduce() {
        let result = reduce("Wie/Englisch", EXAMPLE_CALENDAR).unwrap();
        let parsed = ParsedCalendar::from(result.as_str());
        assert_ne!(parsed.events.len(), 0);
        assert!(parsed.events.into_iter().all(|x| !x.summary.contains("Wiederholungs") && !x.summary.contains("Englisch")));
    }

    #[test]
    fn test_max_regex_limit() {
        assert!(reduce("1/2/3/4/5/6/7/8/9", EXAMPLE_CALENDAR).is_ok());
        assert!(reduce("1/2/3/4/5/6/7/8/9/10", EXAMPLE_CALENDAR).is_err());
    }
}
