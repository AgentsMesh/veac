use std::path::Path;

use veac_ir::RationalTime;

use super::FramePixelFormat;
use crate::RuntimeError;

pub(crate) fn frame(
    path: &Path,
    stream: u32,
    time: RationalTime,
    format: FramePixelFormat,
) -> Result<Vec<String>, RuntimeError> {
    let mut output = base(path)?;
    output.extend([
        "-map".to_owned(),
        format!("0:v:{stream}"),
        "-loglevel".to_owned(),
        "info".to_owned(),
    ]);
    let trim = format!("trim=start={}", seconds(time));
    let filter = match format {
        FramePixelFormat::Alpha16 => format!("{trim},alphaextract,showinfo"),
        FramePixelFormat::Rgb8 | FramePixelFormat::Rgba8 => format!("{trim},showinfo"),
    };
    output.extend(["-vf".to_owned(), filter]);
    output.extend([
        "-frames:v".to_owned(),
        "1".to_owned(),
        "-pix_fmt".to_owned(),
        format.ffmpeg_name().to_owned(),
        "-f".to_owned(),
        "rawvideo".to_owned(),
        "pipe:1".to_owned(),
    ]);
    Ok(output)
}

pub(crate) fn decode(path: &Path, stream: u32) -> Result<Vec<String>, RuntimeError> {
    let mut output = base(path)?;
    output.extend([
        "-map".to_owned(),
        format!("0:v:{stream}"),
        "-progress".to_owned(),
        "pipe:1".to_owned(),
        "-nostats".to_owned(),
        "-f".to_owned(),
        "null".to_owned(),
        "-".to_owned(),
    ]);
    Ok(output)
}

fn base(path: &Path) -> Result<Vec<String>, RuntimeError> {
    let path = path
        .to_str()
        .ok_or_else(|| RuntimeError::new("media observation path is not valid UTF-8"))?;
    let mut output = vec![
        "-hide_banner".to_owned(),
        "-loglevel".to_owned(),
        "error".to_owned(),
        "-nostdin".to_owned(),
        "-noautorotate".to_owned(),
    ];
    output.extend(crate::input_policy::string_arguments());
    output.extend([
        "-i".to_owned(),
        path.to_owned(),
        "-threads".to_owned(),
        "1".to_owned(),
    ]);
    Ok(output)
}

fn seconds(value: RationalTime) -> String {
    let scale = i128::from(value.timescale);
    let raw = i128::from(value.value);
    let whole = raw / scale;
    let mut remainder = raw % scale;
    let mut fraction = String::with_capacity(9);
    for _ in 0..9 {
        remainder *= 10;
        fraction.push(char::from(
            b'0' + u8::try_from(remainder / scale).unwrap_or(0),
        ));
        remainder %= scale;
    }
    while fraction.ends_with('0') && fraction.len() > 1 {
        fraction.pop();
    }
    format!("{whole}.{fraction}")
}

#[cfg(test)]
#[path = "command/tests.rs"]
mod tests;
