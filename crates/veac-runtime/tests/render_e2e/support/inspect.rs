use std::process::Command;

use serde_json::Value;

use super::*;

pub(crate) fn assert_media_contract(media: &Path, audio_streams: usize, duration: f64) {
    let value = ffprobe(media);
    let streams = value["streams"].as_array().expect("ffprobe streams");
    let videos: Vec<_> = streams
        .iter()
        .filter(|stream| stream["codec_type"] == "video")
        .collect();
    let audios: Vec<_> = streams
        .iter()
        .filter(|stream| stream["codec_type"] == "audio")
        .collect();
    assert_eq!(videos.len(), 1, "output must have one video stream");
    assert_eq!(audios.len(), audio_streams, "unexpected audio streams");
    let video = videos[0];
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(video["width"], WIDTH);
    assert_eq!(video["height"], HEIGHT);
    assert_eq!(video["r_frame_rate"], format!("{FPS}/1"));
    if let Some(audio) = audios.first() {
        assert_eq!(audio["codec_name"], "aac");
        assert_eq!(audio["sample_rate"], "48000");
        assert_eq!(audio["channels"], 1);
    }
    let actual = value["format"]["duration"]
        .as_str()
        .expect("format duration")
        .parse::<f64>()
        .expect("numeric duration");
    assert!((actual - duration).abs() <= 0.12, "duration {actual}");
}

pub(crate) fn rgb_at(media: &Path, second: f64, x: u32, y: u32) -> [u8; 3] {
    let filter = format!("crop=2:2:{x}:{y},scale=1:1,format=rgb24");
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(media)
        .arg("-ss")
        .arg(second.to_string())
        .args(["-vf", &filter, "-frames:v", "1", "-f", "rawvideo", "-"])
        .output()
        .expect("extract output pixel");
    assert!(
        output.status.success() && output.stdout.len() >= 3,
        "pixel extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    [output.stdout[0], output.stdout[1], output.stdout[2]]
}

pub(crate) fn video_frame_count(media: &Path) -> u64 {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-count_frames",
            "-show_entries",
            "stream=nb_read_frames",
            "-of",
            "default=nokey=1:noprint_wrappers=1",
        ])
        .arg(media)
        .output()
        .expect("count output frames");
    assert!(output.status.success(), "ffprobe frame count failed");
    String::from_utf8(output.stdout)
        .expect("UTF-8 frame count")
        .trim()
        .parse()
        .expect("numeric frame count")
}

pub(crate) fn rgb_frame(media: &Path, second: f64) -> Vec<u8> {
    rgb_frame_sized(media, second, WIDTH, HEIGHT)
}

pub(crate) fn rgb_frame_sized(media: &Path, second: f64, width: u32, height: u32) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(media)
        .arg("-ss")
        .arg(second.to_string())
        .args([
            "-vf",
            "format=rgb24",
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .expect("extract RGB frame");
    assert!(
        output.status.success(),
        "frame extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), (width * height * 3) as usize);
    output.stdout
}

pub(crate) fn audio_samples(media: &Path, start: f64, duration: f64) -> Vec<f64> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(media)
        .arg("-ss")
        .arg(start.to_string())
        .arg("-t")
        .arg(duration.to_string())
        .args([
            "-map", "0:a:0", "-ac", "1", "-ar", "8000", "-f", "s16le", "-",
        ])
        .output()
        .expect("extract PCM window");
    assert!(
        output.status.success(),
        "PCM extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .chunks_exact(2)
        .map(|bytes| f64::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.0)
        .collect()
}

pub(crate) fn rms_db(samples: &[f64]) -> f64 {
    let mean = samples.iter().map(|value| value * value).sum::<f64>() / samples.len() as f64;
    if mean <= f64::EPSILON {
        -120.0
    } else {
        20.0 * mean.sqrt().log10()
    }
}

pub(crate) fn tone_power(samples: &[f64], frequency: f64) -> f64 {
    let (real, imaginary) =
        samples
            .iter()
            .enumerate()
            .fold((0.0, 0.0), |(real, imaginary), (index, sample)| {
                let phase = std::f64::consts::TAU * frequency * index as f64 / 8000.0;
                (
                    real + sample * phase.cos(),
                    imaginary - sample * phase.sin(),
                )
            });
    (real * real + imaginary * imaginary).sqrt() / samples.len() as f64
}

fn ffprobe(media: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name,width,height,r_frame_rate,sample_rate,channels:format=duration",
            "-of",
            "json",
        ])
        .arg(media)
        .output()
        .expect("run output ffprobe");
    assert!(
        output.status.success(),
        "output ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("ffprobe JSON")
}
