use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::consts::STUNDENPLAN_URL;

pub type CourseMap = RwLock<HashMap<String, Vec<Semester>>>;

lazy_static! {
    static ref COURSE_REGEX: Regex = Regex::new(r#"<option value=".*?"[^>]*?>(.*?)</option>"#).unwrap();
    static ref SEMESTER_PARENT_REGEX: Regex = Regex::new(r#"\[()\]|,\[(\[.*?\])\]"#).unwrap();
    static ref SEMESTER_LITERAL_REGEX: Regex = Regex::new(r#""(.*?)""#).unwrap();
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
        let res = reqwest::get(STUNDENPLAN_URL).await?;

        if !res.status().is_success() {
            return Err(anyhow!("status code was {}", &res.status().as_str()));
        }

        let content = res.text().await?;

        let courses = parse_courses(&content)?;
        let semester = parse_semester(&content)?;

        if courses.len() != semester.len() {
            return Err(anyhow!("course and semester length differ"));
        }

        let mut course_guard = self.course.write().await;
        course_guard.clear();
        for (course, semester) in courses.into_iter().zip(semester.into_iter()) {
            // Skip courses that start with a . like .gnupg
            if course.starts_with(".") {
                continue;
            }

            // Skip courses whose semester does not have the necessary " - " divisor
            // just hwr stuff...
            if semester.iter().any(|s| !s.contains(" - ")) {
                continue;
            }

            let mappedSemester = semester.into_iter().map(|s| {
                let split: Vec<&str> = s.split(" - ").collect();
                if split.len() != 2 {
                    return Err(anyhow!("invalid semester format"));
                }

                Ok(
                    Semester {
                        display_name: s.to_owned(),
                        year_part: split[0].to_owned(),
                        course_part: split[1].to_owned()
                    }
                )
            }).collect::<Result<Vec<Semester>, anyhow::Error>>()?;

            course_guard.insert(course.to_owned(), mappedSemester);
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
                    Err(e) =>  tracing::error!(error = %e, "error while fetching")
                }
            }
        }
    });
}

fn parse_courses(body: &str) -> anyhow::Result<Vec<&str>> {
    let mut courses = Vec::new();

    for capture in COURSE_REGEX.captures_iter(body) {
        courses.push(capture.get(1).ok_or_else(|| anyhow!("match has no course"))?.as_str());
    }

    Ok(courses)
}

fn parse_semester(body: &str) -> anyhow::Result<Vec<Vec<&str>>> {
    let mut result = Vec::new();

    for capture in SEMESTER_PARENT_REGEX.captures_iter(body) {
        let matched = capture.get(0).unwrap();

        let mut childVec = Vec::new();

        for child in SEMESTER_LITERAL_REGEX.captures_iter(matched.as_str()) {
            let mut course = child.get(1).ok_or_else(|| anyhow!("no string in match found"))?.as_str();
            if course.contains(".") {
                // strip stuff like .html
                course = course.split(".").collect::<Vec<&str>>()[0];
            }
            childVec.push(course);
        }

        result.push(childVec);
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_STUNDENPLAN: &str = include_str!("../example_stundenplan.php");

    #[test]
    fn test_parse_courses() {
        let result = parse_courses(&EXAMPLE_STUNDENPLAN).unwrap();

        assert_eq!(result[0], ".gnupg");
        assert_eq!(result[1], "IP");
        assert_eq!(result[2], "bank");
        assert_eq!(result[3], "bauwesen");
        //
        // skip the in-between
        //
        assert_eq!(result[19], "andere");
    }

    #[test]
    fn test_parse_semester() {
        let result = parse_semester(&EXAMPLE_STUNDENPLAN).unwrap();

        assert_eq!(result[0].len(), 0);
        assert_eq!(result[1].len(), 6);
        assert_eq!(result[19][0], "6B_151_153");
        assert_eq!(result[19][1], "Drebing_Michael");
        assert_eq!(result[19][2], "Gapp");
        assert_eq!(result[19][3], "Incomings_DLM");
        assert_eq!(result[19][21], "wannemacher");
    }
}