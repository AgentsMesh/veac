use veac_ir::{Rational, RationalTime};

use crate::RuntimeError;

pub(super) fn frame_pts(stderr: &[u8], time_base: Rational) -> Result<RationalTime, RuntimeError> {
    let text = String::from_utf8_lossy(stderr);
    let raw = text
        .lines()
        .filter(|line| line.contains("showinfo") && line.contains(" pts:"))
        .filter_map(|line| field(line, "pts:"))
        .next()
        .ok_or_else(|| RuntimeError::new("FFmpeg did not report the selected frame PTS"))?;
    let pts = raw
        .parse::<i64>()
        .map_err(|_| RuntimeError::new("FFmpeg reported an invalid selected frame PTS"))?;
    let value = pts
        .checked_mul(time_base.numerator)
        .ok_or_else(|| RuntimeError::resource_limit("selected frame PTS overflowed"))?;
    RationalTime::new(value, time_base.denominator)
        .map_err(|_| RuntimeError::new("selected frame PTS is outside the exact time domain"))
}

pub(super) fn decode_progress(stdout: &[u8]) -> Result<(u64, Option<RationalTime>), RuntimeError> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| RuntimeError::new("FFmpeg decode progress was not valid UTF-8"))?;
    let frames = values(text, "frame")
        .filter_map(|value| value.parse::<u64>().ok())
        .max()
        .ok_or_else(|| RuntimeError::new("FFmpeg did not report decoded frame progress"))?;
    let micros = values(text, "out_time_us")
        .filter_map(|value| value.parse::<i64>().ok())
        .max();
    let last_pts = micros
        .map(|value| RationalTime::new(value, 1_000_000))
        .transpose()
        .map_err(|_| RuntimeError::new("FFmpeg decode progress time is invalid"))?;
    Ok((frames, last_pts))
}

fn values<'a>(text: &'a str, key: &'a str) -> impl Iterator<Item = &'a str> {
    text.lines()
        .filter_map(move |line| line.split_once('='))
        .filter_map(move |(name, value)| (name == key).then_some(value.trim()))
}

fn field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let tail = line.split_once(name)?.1.trim_start();
    tail.split_ascii_whitespace().next()
}

#[cfg(test)]
#[path = "parse/tests.rs"]
mod tests;
