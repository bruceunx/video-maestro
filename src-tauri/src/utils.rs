use crate::whisper::Segment;
use regex::Regex;
use std::time::Duration;
use tube_rs::SubtitleEntry;

struct TimelineEntry {
    timestamp: Duration,
    content: String,
}

fn parse_timeline(content: &str) -> Vec<TimelineEntry> {
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
                    content: format!("[{minutes:02}:{seconds:02} - {description}] \n"),
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

    let mut timelines = parse_timeline(description);
    timelines.sort_by_key(|e| e.timestamp);

    if timelines.len() > 0 {
        for i in 0..timelines.len() {
            let current_start = timelines[i].timestamp.as_secs_f64() * 1000.0;
            let current_end = if i < timelines.len() - 1 {
                timelines[i + 1].timestamp.as_secs_f64() * 1000.0
            } else {
                segments.last().map_or(current_start + 60.0, |s| s.end) * 1000.0
            };

            let relevant_segments: Vec<&Segment> = segments
                .iter()
                .filter(|segment| {
                    (segment.start >= current_start && segment.start < current_end)
                        || (segment.end > current_start && segment.end <= current_end)
                        || (segment.start <= current_start && segment.end >= current_end)
                })
                .collect();

            timelines[i].content.push_str(
                relevant_segments
                    .iter()
                    .map(|segment| segment.text.trim())
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .as_ref(),
            );
        }

        for timeline in timelines {
            if current_string.len() + timeline.content.len() > 3000 {
                chunks.push(current_string.clone());
                current_string.clear();
            };

            current_string.push_str(&timeline.content);
        }

        return chunks;
    }

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
    }

    #[test]
    fn test_with_dash() {
        let input = "01:23 - With dash";
        let result = parse_timeline(input);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_with_em_dash() {
        let input = "02:45 — With em dash";
        let result = parse_timeline(input);
        assert_eq!(result.len(), 1);
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
