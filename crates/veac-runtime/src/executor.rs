use std::process::Command;

use veac_codegen::ffmpeg::FfmpegCommand;

use crate::RuntimeError;

/// Execute an FFmpeg command as a subprocess.
pub fn execute(cmd: &FfmpegCommand) -> Result<(), RuntimeError> {
    let args = cmd.to_args();
    execute_ffmpeg(&args)
}

/// Execute FFmpeg with the given argument list.
pub fn execute_ffmpeg(args: &[String]) -> Result<(), RuntimeError> {
    check_ffmpeg()?;

    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|e| RuntimeError::new(format!("failed to run ffmpeg: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RuntimeError::new(format!("ffmpeg failed:\n{stderr}")));
    }

    Ok(())
}

/// Verify that FFmpeg is installed and return its version string.
pub fn check_ffmpeg() -> Result<String, RuntimeError> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| {
            RuntimeError::new("ffmpeg not found. Install FFmpeg: https://ffmpeg.org/download.html")
        })?;

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("unknown").to_string();
    Ok(first_line)
}
