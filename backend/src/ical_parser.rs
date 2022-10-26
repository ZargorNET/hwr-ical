use fancy_regex::{Captures, Regex};
use lazy_static::lazy_static;

const EVENT_REGEX: &str = r#"BEGIN:VEVENT\n(?:(?:.|\n)*?)\nSUMMARY:(?P<summary>(?:.|\n)*?)(?=\n[^\t])(?:(?:.|\n)*?)\nEND:VEVENT"#;

lazy_static! {
    static ref COMPILED_REGEX: Regex = Regex::new(EVENT_REGEX).unwrap();
}

pub struct ParsedCalendar<'a> {
    pub events: Vec<ParsedEvent>,
    pub raw: &'a str,
}


pub struct ParsedEvent {
    pub start: usize,
    pub end: usize,
    pub summary: String,
}

impl<'a> From<&'a str> for ParsedCalendar<'a> {
    fn from(s: &'a str) -> Self {
        let captures: Vec<Captures> = COMPILED_REGEX.captures_iter(s)
            .flat_map(|c| c.ok()).collect();

        let mut events = Vec::with_capacity(captures.len());
        for capture in captures {
            let first = capture.get(0).unwrap();
            let summary = capture.name("summary").unwrap();

            events.push(ParsedEvent {
                start: first.start(),
                end: first.end(),
                summary: summary.as_str().replace("\t", "").replace("\n", ""),
            });
        }

        ParsedCalendar { events, raw: s }
    }
}
