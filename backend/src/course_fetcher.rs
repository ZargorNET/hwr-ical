use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use chrono::{TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::consts::{MOODLE_LOGIN_URL, STUNDENPLAN_URL};

pub type CourseMap = RwLock<HashMap<String, Vec<Semester>>>;

lazy_static! {
    static ref LOGINTOKEN_REGEX: Regex =
        Regex::new(r##"<input[^>]*name="logintoken"[^>]*value="([^"]*)"[^>]*>"##).unwrap();
    static ref COURSE_REGEX: Regex =
        Regex::new(r#"<option value=".*?"[^>]*?>(.*?)</option>"#).unwrap();
    static ref KURSE_JSON_REGEX: Regex = Regex::new(r#"var kurse\s*=\s*(\[[\s\S]*?\]);"#).unwrap();
}

#[derive(Serialize)]
pub struct Semester {
    pub display_name: String,
    pub year_part: String,
    pub course_part: String,
}

pub struct CourseFetcher {
    pub course: CourseMap, // Course, Semester
}

impl CourseFetcher {
    async fn fetch(&self) -> anyhow::Result<()> {
        let client = reqwest::ClientBuilder::new().cookie_store(true).build()?;
        let res = client.get(STUNDENPLAN_URL).send().await?;

        if !res.status().is_success() {
            return Err(anyhow!("status code was {}", &res.status().as_str()));
        }

        let mut content = res.text().await?;

        // They added a guest login with CSRF...
        // Quick and dirty way to fix that.
        if content.contains("logintoken") {
            let logintoken = LOGINTOKEN_REGEX
                .captures(&content)
                .context("no logintoken found")?
                .get(1)
                .context("group not found")?
                .as_str();

            let res = client
                .post(MOODLE_LOGIN_URL)
                .form(&HashMap::from([
                    ("logintoken", logintoken),
                    ("username", "guest"),
                    ("password", "guest"),
                ]))
                .send()
                .await?;

            if !res.status().is_success() {
                return Err(anyhow!("login status code was {}", &res.status().as_str()));
            }

            // Resend request after login.
            // The auth token is in the cookie jar.
            let res = client.get(STUNDENPLAN_URL).send().await?;
            content = res.text().await?;
        }

        let courses = parse_courses(&content)?;
        let semester = parse_semester(&content)?;

        if courses.len() != semester.len() {
            return Err(anyhow!("course and semester length differ"));
        }

        let mut course_guard = self.course.write().await;
        course_guard.clear();
        for (course, semester) in courses.into_iter().zip(semester) {
            // Skip courses that start with a . like .gnupg
            if course.starts_with(".") {
                continue;
            }

            // Skip courses whose semester does not have the necessary " - " divisor
            // just hwr stuff...
            if semester.iter().any(|s| !s.contains(" - ")) {
                continue;
            }

            let mapped_semester = semester
                .into_iter()
                .map(|s| {
                    let split: Vec<&str> = s.split(" - ").collect();
                    if split.len() != 2 {
                        return Err(anyhow!("invalid semester format"));
                    }

                    Ok(Semester {
                        display_name: s.to_owned(),
                        year_part: split[0].to_owned(),
                        course_part: split[1].to_owned(),
                    })
                })
                .collect::<Result<Vec<Semester>, anyhow::Error>>()?;

            course_guard.insert(course.to_owned(), mapped_semester);
        }

        Ok(())
    }
}

pub fn start(fetcher: Arc<CourseFetcher>) {
    tokio::spawn(async move {
        let mut in24h = Utc.timestamp(0, 0);
        let mut timer = tokio::time::interval(Duration::from_secs(10 * 60));

        loop {
            timer.tick().await;
            if Utc::now() < in24h {
                continue;
            }

            in24h = Utc::now() + chrono::Duration::hours(24);

            tracing::info!(next_fetch = %in24h, "fetching courses");

            loop {
                match fetcher.fetch().await {
                    Ok(_) => break,
                    Err(e) => tracing::error!(error = %e, "error while fetching"),
                }
            }
        }
    });
}

fn parse_courses(body: &str) -> anyhow::Result<Vec<&str>> {
    let mut courses = Vec::new();

    for capture in COURSE_REGEX.captures_iter(body) {
        let course = capture
            .get(1)
            .ok_or_else(|| anyhow!("match has no course"))?
            .as_str();

        if course == "Fachrichtung wählen" {
            continue;
        }

        // Refer to the example_stundenplan.php.
        // If we encounter this, the remaining option fields are unwanted.
        if course == "Kurs Wählen" {
            break;
        }

        courses.push(course);
    }

    Ok(courses)
}

fn parse_semester(body: &str) -> anyhow::Result<Vec<Vec<String>>> {
    let caps = KURSE_JSON_REGEX
        .captures(body)
        .ok_or_else(|| anyhow!("could not find var kurse in body"))?;

    let json_str = caps.get(1).unwrap().as_str();
    let parsed: Vec<Vec<Vec<String>>> = serde_json::from_str(json_str)?;

    let mut result = Vec::new();
    for course_semesters in parsed {
        let mut child_vec = Vec::new();
        for sem in course_semesters {
            if let Some(mut course) = sem.into_iter().next() {
                if course.contains(".") {
                    // strip stuff like .html
                    if let Some(first_part) = course.split(".").next() {
                        course = first_part.to_string();
                    }
                }
                child_vec.push(course);
            }
        }
        result.push(child_vec);
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_STUNDENPLAN: &str = include_str!("../testdata/example_stundenplan.php");

    #[test]
    fn test_parse_courses() {
        let result = parse_courses(EXAMPLE_STUNDENPLAN).unwrap();

        assert_eq!(result[0], "IP");
        assert_eq!(result[1], "bank");
        assert_eq!(result[2], "bauwesen");
        //
        // skip the in-between
        //
        assert_eq!(result[19], "andere");
    }

    #[test]
    fn test_parse_semester() {
        let result = parse_semester(EXAMPLE_STUNDENPLAN).unwrap();

        dbg!(&result);
        assert_eq!(
            result[1],
            [
                "semester1 - kursa",
                "semester1 - kursb",
                "semester1 - kursc",
                "semester2 - kursa",
                "semester2 - kursb",
                "semester2 - kursc",
                "semester3 - kursa",
                "semester3 - kursb",
                "semester3 - kursc",
                "semester4 - kursa",
                "semester4 - kursb",
                "semester4 - kursc",
                "semester5 - kursa",
                "semester5 - kursb",
                "semester5 - kursc",
                "semester6 - kursa",
                "semester6 - kursb",
                "semester6 - kursc"
            ]
        );
        assert_eq!(result[19][0], "6B_151_153");
        assert_eq!(result[19][1], "Drebing_Michael");
        assert_eq!(result[19][2], "Gapp");
        assert_eq!(result[19][3], "IP21_5_5");
    }
}
