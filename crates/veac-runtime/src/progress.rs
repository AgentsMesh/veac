/// Parsed FFmpeg progress information from stderr output.
#[derive(Debug, Clone)]
pub struct RenderProgress {
    pub frame: u64,
    pub fps: f64,
    pub time_sec: f64,
    pub speed: f64,
}

/// Parse an FFmpeg stderr progress line.
///
/// Example line:
/// `frame=  100 fps=30.0 ... time=00:00:03.33 ... speed=1.00x`
pub fn parse_ffmpeg_progress(line: &str) -> Option<RenderProgress> {
    if !line.contains("frame=") || !line.contains("time=") {
        return None;
    }

    let frame = extract_value(line, "frame=")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let fps = extract_value(line, "fps=")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0);

    let time_sec = extract_value(line, "time=")
        .map(|v| parse_time_to_seconds(v.trim()))
        .unwrap_or(0.0);

    let speed = extract_value(line, "speed=")
        .and_then(|v| v.trim().trim_end_matches('x').parse::<f64>().ok())
        .unwrap_or(0.0);

    Some(RenderProgress {
        frame,
        fps,
        time_sec,
        speed,
    })
}

/// Extract the value after `key=` up to the next space or `=`-prefixed key.
fn extract_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    // Value ends at the next key pattern (word followed by =) or end of string.
    let end = rest
        .find(|c: char| c.is_ascii_alphabetic())
        .map(|pos| {
            // Check if this is the start of a next key (has = after alpha chars)
            let after = &rest[pos..];
            if after.contains('=')
                && after.find('=').unwrap() < after.find(' ').unwrap_or(after.len())
            {
                pos
            } else {
                rest.len()
            }
        })
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

/// Parse HH:MM:SS.ms format to seconds.
fn parse_time_to_seconds(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let s: f64 = parts[2].parse().unwrap_or(0.0);
        h * 3600.0 + m * 60.0 + s
    } else {
        time_str.parse().unwrap_or(0.0)
    }
}
