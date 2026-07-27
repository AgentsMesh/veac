use std::fmt::Write;

use veac_plan::canonical::CaptionSidecarFormat;

use super::{failure::Failure, format::timestamp, Cue};

pub(super) fn render(format: CaptionSidecarFormat, cues: &[Cue<'_>]) -> Result<String, Failure> {
    match format {
        CaptionSidecarFormat::Srt => srt(cues),
        CaptionSidecarFormat::WebVtt => webvtt(cues),
        CaptionSidecarFormat::Ass => unreachable!("ASS uses its fidelity renderer"),
    }
}

fn srt(cues: &[Cue<'_>]) -> Result<String, Failure> {
    let mut output = String::new();
    for (index, cue) in cues.iter().enumerate() {
        validate_rich(cue, "SRT")?;
        if cue.speaker.is_some() {
            return Err(unsupported(
                cue,
                "CAPTION_SRT_SPEAKER_UNSUPPORTED",
                "SRT has no cue-level speaker field",
            ));
        }
        let text = safe_text(cue)?;
        let _ = writeln!(
            output,
            "{}\n{} --> {}\n{}\n",
            index + 1,
            timestamp(cue.start, ','),
            timestamp(cue.end, ','),
            text
        );
    }
    Ok(output)
}

fn webvtt(cues: &[Cue<'_>]) -> Result<String, Failure> {
    let mut output = "WEBVTT\n\n".to_owned();
    for cue in cues {
        validate_rich(cue, "WebVTT")?;
        let mut text = html(&safe_text(cue)?);
        if let Some(speaker) = cue.speaker {
            text = format!("<v {}>{text}</v>", html(speaker));
        }
        let _ = writeln!(
            output,
            "{} --> {}\n{}\n",
            timestamp(cue.start, '.'),
            timestamp(cue.end, '.'),
            text
        );
    }
    Ok(output)
}

fn validate_rich(cue: &Cue<'_>, format: &str) -> Result<(), Failure> {
    if cue.content.style.spans.is_empty() {
        Ok(())
    } else {
        Err(unsupported(
            cue,
            "CAPTION_RICH_TEXT_UNSUPPORTED",
            format!("{format} delivery cannot preserve resolved rich-text spans"),
        ))
    }
}

fn safe_text(cue: &Cue<'_>) -> Result<String, Failure> {
    if cue.content.text.contains('\0') {
        return Err(invalid(cue, "caption text contains NUL"));
    }
    let text = cue.content.text.replace("\r\n", "\n").replace('\r', "\n");
    if text.starts_with('\n') || text.ends_with('\n') || text.contains("\n\n") {
        return Err(invalid(
            cue,
            "caption text contains a blank cue-separator line",
        ));
    }
    Ok(text)
}

fn html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unsupported(cue: &Cue<'_>, code: &'static str, message: impl Into<String>) -> Failure {
    Failure::unsupported(code, message).at(&cue.clip.id)
}

fn invalid(cue: &Cue<'_>, message: &str) -> Failure {
    Failure::invalid("CAPTION_TEXT_UNSAFE", message).at(&cue.clip.id)
}
