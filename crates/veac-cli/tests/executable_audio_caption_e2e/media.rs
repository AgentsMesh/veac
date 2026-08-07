use std::path::Path;
use std::process::Command;

const SAMPLE_RATE: f64 = 48_000.0;

pub fn assert_delivery(path: &Path) {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name,sample_rate,channels,channel_layout:format=duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let audio = value["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .unwrap();
    assert_eq!(audio["codec_name"], "aac");
    assert_eq!(audio["sample_rate"], "48000");
    assert_eq!(audio["channels"], 2);
    assert_eq!(audio["channel_layout"], "stereo");
    let duration = value["format"]["duration"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((3.9..=4.1).contains(&duration));
}

pub fn assert_audio_timing(path: &Path) {
    let before = samples(path, 0.2, 0.4);
    let audible = samples(path, 1.25, 0.5);
    let after = samples(path, 3.3, 0.4);
    let audible_rms = rms(&audible);
    assert!(audible_rms > 0.05, "audible RMS was {audible_rms}");
    assert!(rms(&before) < 0.005, "pre-roll was not silent");
    assert!(rms(&after) < 0.005, "post-roll was not silent");
    let frequency = rising_frequency(&audible);
    assert!(
        (650.0..=670.0).contains(&frequency),
        "frequency={frequency}"
    );
}

pub fn assert_caption_timing(path: &Path) {
    let intro = frame(path, 0.25);
    let audible = frame(path, 2.5);
    let silent = frame(path, 3.5);
    for (label, value) in [
        ("intro", &intro),
        ("audible", &audible),
        ("silent", &silent),
    ] {
        assert!(bright_pixels(value) > 40, "{label} caption was not visible");
    }
    assert!(different_pixels(&intro, &audible) > 100);
    assert!(different_pixels(&audible, &silent) > 100);
}

fn samples(path: &Path, start: f64, duration: f64) -> Vec<f32> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &start.to_string(),
        ])
        .args(["-t", &duration.to_string(), "-i"])
        .arg(path)
        .args(["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}

fn rms(samples: &[f32]) -> f64 {
    let power = samples
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    power.sqrt()
}

fn rising_frequency(samples: &[f32]) -> f64 {
    let crossings = samples
        .windows(2)
        .filter(|pair| pair[0] <= 0.0 && pair[1] > 0.0)
        .count();
    crossings as f64 * SAMPLE_RATE / (samples.len() - 1) as f64
}

fn frame(path: &Path, second: f64) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-ss", &second.to_string(), "-frames:v", "1"])
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), 640 * 360 * 3);
    output.stdout
}

fn bright_pixels(frame: &[u8]) -> usize {
    frame
        .chunks_exact(3)
        .filter(|pixel| pixel.iter().all(|channel| *channel > 150))
        .count()
}

fn different_pixels(left: &[u8], right: &[u8]) -> usize {
    left.chunks_exact(3)
        .zip(right.chunks_exact(3))
        .filter(|(left, right)| {
            left.iter()
                .zip(right.iter())
                .map(|(a, b)| a.abs_diff(*b) as usize)
                .sum::<usize>()
                > 90
        })
        .count()
}
