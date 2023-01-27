use std::collections::HashMap;
use std::str::FromStr;
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::Response;
use encoding_rs::UTF_8;
use icalendar::{Calendar, CalendarComponent, Component, EventLike};
use regex::{Regex, RegexBuilder};

use crate::AppError;
use crate::consts::{MAX_REGEX_COUNT, REVERSE_PROXY_URL};

pub async fn root(Path((study, semester, course, rregex)): Path<(String, String, String, String)>, Query(query): Query<HashMap<String, String>>) -> Result<Response<Body>, AppError> {
    let regex = &rregex[1..];

    let res = reqwest::get(REVERSE_PROXY_URL.to_owned() + &format!("{study}/{semester}/{course}")).await.unwrap();
    let status = res.status();
    let body = res.bytes().await.unwrap();
    let res = UTF_8.decode_with_bom_removal(&body).0.to_string();
    if !status.is_success() || res.contains(r#"alert("Fehlerhafter Pfad!");"#) {
        return Err(anyhow!("proxy error. is your path correct?").into());
    }

    let mut calendar = Calendar::from_str(&res).map_err(|e| anyhow!(e))?;

    if !regex.is_empty() {
        reduce(regex, &mut calendar)?;
    }

    let new_format = bool::from_str(query.get("new").unwrap_or(&"true".to_owned())).unwrap_or(true);
    if new_format {
        simplify(&mut calendar);
    }

    Ok(
        Response::builder()
            .header("Content-Disposition", "inline; filename=calendar.ics")
            .header("Content-Type", "text/calendar; charset=utf-8")
            .status(StatusCode::OK)
            .body(Body::from(calendar.to_string())).unwrap()
    )
}

fn reduce(regex: &str, calendar: &mut Calendar) -> anyhow::Result<()> {
    let regex_split: Vec<&str> = regex.split("/").collect();

    if regex_split.len() > MAX_REGEX_COUNT as usize {
        return Err(anyhow!("too many regex").into());
    }

    let built_regex: Vec<Regex> = regex_split
        .iter()
        .filter(|r| !r.is_empty())
        .filter_map(|r| RegexBuilder::new(("(?i)".to_owned() + &r).as_str()).size_limit(100000).build().ok())
        .collect();

    calendar.components.retain(|component| {
        if let Some(event) = component.as_event() {
            let Some(summary) = event.get_summary() else { return true; };

            return if built_regex.iter().any(|r| r.is_match(&summary)) {
                false
            } else {
                true
            };
        }

        true
    });

    Ok(())
}

fn simplify(calendar: &mut Calendar) {
    for component in calendar.components.iter_mut() {
        if let CalendarComponent::Event(ref mut event) = component {
            let Some(summary) = event.get_summary() else { continue; };
            let summary = summary.replace("\\", "");

            let mut split = summary.split(r#";"#);


            let Some(period_type) = split.next() else { continue; };

            // "IT2141-Labor SWE II:" to "Labor SWE II:"
            let period = {
                let Some(period) = split.next() else { continue; };
                let split = period.split("-").collect::<Vec<_>>();
                if split.len() == 1 {
                    split.join("")
                } else {
                    split.into_iter().skip(1).collect::<String>()
                }
            };

            let Some(location) = event.get_location() else { continue; };

            let rest = split.collect::<Vec<_>>();


            let extra = {
                let fold_function = |acc, s: &&str| acc + s.to_owned();

                if rest.len() == 3 && rest.get(1).unwrap().starts_with("CL:") {
                    rest.iter().skip(2).fold(String::new(), fold_function)
                } else if rest.len() > 1 {
                    rest.iter().skip(1).fold(String::new(), fold_function)
                } else {
                    rest.iter().fold(String::new(), fold_function)
                }
            };

            let mut new_summary = format!("{}: {}", period_type, period);


            if !extra.is_empty() {
                new_summary.push_str(&format!(" ({})", &extra));
            }


            if !location.is_empty() {
                new_summary.push_str(&format!(" [{}]", location));
            }

            event.summary(&new_summary);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_CALENDAR: &'static str = include_str!("../../testdata/example_calendar.ics");

    #[test]
    fn test_parsing() {
        let calendar = Calendar::from_str(EXAMPLE_CALENDAR).unwrap();
        assert_ne!(calendar.components.iter().filter(|e| matches!(e, CalendarComponent::Event(_))).count(), 0);
    }

    #[test]
    fn test_simplify() {
        let mut calendar = Calendar::from_str(EXAMPLE_CALENDAR).unwrap();
        simplify(&mut calendar);

        let filtered = calendar.components.into_iter().filter_map(|e| match e {
            CalendarComponent::Event(e) => Some(e),
            _ => None
        }).collect::<Vec<_>>();

        assert_eq!(filtered.get(0).unwrap().get_summary().unwrap(), "Klausur: BWL (Nach- u. Wiederholungsklausur) [CL: 6B.452 (T)]");
        assert_eq!(filtered.get(1).unwrap().get_summary().unwrap(), "SU: Embedded Systems (Online)");
        assert_eq!(filtered.get(2).unwrap().get_summary().unwrap(), "SU: IT und Gesellschaft [CL: 6B.173 (T)]");
        assert_eq!(filtered.get(18).unwrap().get_summary().unwrap(), "PÜ: Labor Embedded Systems (Ausweichtermin) [CL: 6B.158 (E-L)]");
        assert_eq!(filtered.get(filtered.len() - 2).unwrap().get_summary().unwrap(), "Klausur: Embedded Systems (PRÄSENZ, Klausurraumplanung Studiendekanat)");
        assert_eq!(filtered.last().unwrap().get_summary().unwrap(),
                   "Klausur: Embedded Systems (Schreibzeitverlängerung nur für Matr.Nrn.: 699727, 693733 (50%) 676322 (25%), 645463 (25%+DINA3-Ausdruck))");
    }

    #[test]
    fn test_reduce() {
        let mut calendar = Calendar::from_str(EXAMPLE_CALENDAR).unwrap();

        reduce("Wie/Englisch", &mut calendar).unwrap();

        let events = calendar.components.iter().filter(|e| matches!(e, CalendarComponent::Event(_)))
            .map(|e| e.as_event().unwrap()).collect::<Vec<_>>();

        assert!(events.iter().map(|e| e.get_summary().unwrap()).all(|s| !s.contains("Wiederholungs") && !s.contains("Englisch")));
        assert_ne!(events.len(), 0);
    }
}
