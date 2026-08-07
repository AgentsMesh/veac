use std::fmt::Write;

use veac_plan::canonical::{
    CaptionNativeCue, CaptionSidecarFormat, WebVttCueSettings, WebVttTextAlign, WebVttVertical,
};
use veac_plan::ResolvedTextPresentation;

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
            srt_index(cue, index + 1),
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
        if let Some(identifier) = webvtt_identifier(cue) {
            let _ = writeln!(output, "{}", identifier.0);
        }
        let settings = webvtt_settings(cue)
            .map(format_settings)
            .unwrap_or_default();
        let _ = writeln!(
            output,
            "{} --> {}{}\n{}\n",
            timestamp(cue.start, '.'),
            timestamp(cue.end, '.'),
            settings,
            text
        );
    }
    Ok(output)
}

fn srt_index(cue: &Cue<'_>, fallback: usize) -> u64 {
    match &cue.semantics.native {
        Some(CaptionNativeCue::Srt { index }) => *index,
        _ => fallback as u64,
    }
}

fn webvtt_identifier<'a>(cue: &'a Cue<'a>) -> Option<&'a veac_plan::canonical::CaptionNativeId> {
    match &cue.semantics.native {
        Some(CaptionNativeCue::WebVtt { identifier, .. }) => identifier.as_ref(),
        _ => None,
    }
}

fn webvtt_settings<'a>(cue: &'a Cue<'a>) -> Option<&'a WebVttCueSettings> {
    match &cue.semantics.native {
        Some(CaptionNativeCue::WebVtt { settings, .. }) => settings.as_ref(),
        _ => None,
    }
}

fn format_settings(value: &WebVttCueSettings) -> String {
    let mut fields = Vec::new();
    push(&mut fields, "line", value.line.as_deref());
    push(&mut fields, "position", value.position.as_deref());
    push(&mut fields, "size", value.size.as_deref());
    push(&mut fields, "align", value.align.map(align));
    push(&mut fields, "vertical", value.vertical.map(vertical));
    push(
        &mut fields,
        "region",
        value.region.as_ref().map(|region| region.0.as_str()),
    );
    if fields.is_empty() {
        String::new()
    } else {
        format!(" {}", fields.join(" "))
    }
}

fn push(fields: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        fields.push(format!("{name}:{value}"));
    }
}

const fn align(value: WebVttTextAlign) -> &'static str {
    match value {
        WebVttTextAlign::Start => "start",
        WebVttTextAlign::Center => "center",
        WebVttTextAlign::End => "end",
        WebVttTextAlign::Left => "left",
        WebVttTextAlign::Right => "right",
    }
}

const fn vertical(value: WebVttVertical) -> &'static str {
    match value {
        WebVttVertical::Rl => "rl",
        WebVttVertical::Lr => "lr",
    }
}

fn validate_rich(cue: &Cue<'_>, format: &str) -> Result<(), Failure> {
    let has_spans = match &cue.content.presentation {
        ResolvedTextPresentation::Plain { has_spans } => *has_spans,
        ResolvedTextPresentation::Styled { style } => !style.spans.is_empty(),
    };
    if !has_spans {
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
