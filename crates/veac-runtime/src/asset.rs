use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::RuntimeError;

/// Probed media file information.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub duration_sec: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub has_audio: bool,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

/// Probe a media file using ffprobe and return its info.
pub fn probe(path: &Path) -> Result<MediaInfo, RuntimeError> {
    if !path.exists() {
        return Err(RuntimeError::new(format!(
            "file not found: {}",
            path.display()
        )));
    }

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .map_err(|e| RuntimeError::new(format!("failed to run ffprobe: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RuntimeError::new(format!(
            "ffprobe failed on {}: {stderr}",
            path.display()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json(&stdout)
}

fn parse_ffprobe_json(json: &str) -> Result<MediaInfo, RuntimeError> {
    let data: FfprobeOutput = serde_json::from_str(json)
        .map_err(|e| RuntimeError::new(format!("failed to parse ffprobe output: {e}")))?;

    let duration_sec = data
        .format
        .as_ref()
        .and_then(|f| f.duration.as_deref())
        .and_then(|d| d.parse::<f64>().ok())
        .unwrap_or(0.0);

    let mut width = None;
    let mut height = None;
    let mut has_audio = false;
    let mut video_codec = None;
    let mut audio_codec = None;

    for stream in data.streams.unwrap_or_default() {
        match stream.codec_type.as_deref() {
            Some("video") => {
                width = stream.width;
                height = stream.height;
                video_codec = stream.codec_name;
            }
            Some("audio") => {
                has_audio = true;
                audio_codec = stream.codec_name;
            }
            _ => {}
        }
    }

    Ok(MediaInfo {
        duration_sec,
        width,
        height,
        has_audio,
        video_codec,
        audio_codec,
    })
}

impl std::fmt::Display for MediaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Duration:    {:.2}s", self.duration_sec)?;
        if let (Some(w), Some(h)) = (self.width, self.height) {
            writeln!(f, "Resolution:  {w}x{h}")?;
        }
        if let Some(ref vc) = self.video_codec {
            writeln!(f, "Video codec: {vc}")?;
        }
        if self.has_audio {
            if let Some(ref ac) = self.audio_codec {
                writeln!(f, "Audio codec: {ac}")?;
            }
        } else {
            writeln!(f, "Audio:       none")?;
        }
        Ok(())
    }
}

// --- ffprobe JSON structures ---

#[derive(Deserialize)]
struct FfprobeOutput {
    format: Option<FfprobeFormat>,
    streams: Option<Vec<FfprobeStream>>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}
