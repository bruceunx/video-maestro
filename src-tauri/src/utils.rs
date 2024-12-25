use crate::whisper::Segment;
use regex::Regex;
use std::time::Duration;
use tube_rs::SubtitleEntry;

#[derive(Debug)]
struct TimelineEntry {
    timestamp: Duration,
    description: String,
}

impl TimelineEntry {
    fn format_timestamp(&self) -> String {
        format!(
            "{:02}:{:02}",
            self.timestamp.as_secs() / 60,
            self.timestamp.as_secs() % 60
        )
    }

    fn display(&self) -> String {
        format!("{} {}", self.format_timestamp(), self.description)
    }
}

fn parse_timeline(content: &str) -> Vec<TimelineEntry> {
    // Match both formats:
    // "00:00 Description"
    // "00:00 - Description" or "00:00 — Description"
    let re = Regex::new(r"(\d+):(\d{2})(?:\s*[—-])?\s+(.+)").unwrap();

    content
        .lines()
        .filter_map(|line| {
            re.captures(line.trim()).map(|caps| {
                let minutes: u64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);

                let seconds: u64 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);

                let description = caps.get(3).unwrap().as_str().to_string();

                TimelineEntry {
                    timestamp: Duration::from_secs(minutes * 60 + seconds),
                    description,
                }
            })
        })
        .collect()
}

pub fn transform_subtitles_to_segments(subtitles: Vec<SubtitleEntry>) -> Vec<Segment> {
    let mut segments = Vec::new();
    for subtitle in subtitles {
        segments.push(Segment {
            start: subtitle.timestamp as f64,
            end: (subtitle.timestamp + subtitle.duration as u64) as f64,
            text: subtitle.text,
        })
    }
    segments
}

pub fn transform_segments_to_chunks(description: &str, segments: Vec<Segment>) -> Vec<String> {
    let mut chunks = Vec::new();

    let mut current_string = String::new();

    let mut end_time = 0.0;
    for segment in segments {
        if current_string.len() + segment.text.len() > 3000 {
            chunks.push(current_string.clone());
            current_string.clear();
        };

        if current_string.len() + segment.text.len() > 2000 {
            if segment.start - end_time > 7.0 {
                chunks.push(current_string.clone());
                current_string.clear();
            }
        }

        current_string.push_str(&segment.text);
        end_time = segment.end;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_timestamp() {
        let input = "00:00 Simple description";
        let result = parse_timeline(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].format_timestamp(), "00:00");
        assert_eq!(result[0].description, "Simple description");
    }

    #[test]
    fn test_with_dash() {
        let input = "01:23 - With dash";
        let result = parse_timeline(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].format_timestamp(), "01:23");
        assert_eq!(result[0].description, "With dash");
    }

    #[test]
    fn test_with_em_dash() {
        let input = "02:45 — With em dash";
        let result = parse_timeline(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].format_timestamp(), "02:45");
        assert_eq!(result[0].description, "With em dash");
    }

    #[test]
    fn test_multiple_formats() {
        let input = r#"00:00 Direct description
01:23 - Dash description
02:45 — Em dash description"#;
        let result = parse_timeline(input);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_invalid_input() {
        let input = "Invalid timestamp";
        let result = parse_timeline(input);
        assert_eq!(result.len(), 0);
    }
}
