use crate::{CaptionCue, CaptionFormat, InlineStyle, LossReport};

pub(super) fn render(cue: &CaptionCue, format: CaptionFormat, report: &mut LossReport) -> String {
    let characters: Vec<_> = cue.text.plain.chars().collect();
    let mut output = String::new();
    let mut cursor = 0;
    for span in &cue.text.spans {
        output.push_str(&plain(
            &characters[cursor..span.range.start as usize],
            format,
        ));
        let content = plain(
            &characters[span.range.start as usize..span.range.end as usize],
            format,
        );
        output.push_str(&styled(content, &span.style, format, cue, report));
        cursor = span.range.end as usize;
    }
    output.push_str(&plain(&characters[cursor..], format));
    if format == CaptionFormat::WebVtt {
        if let Some(speaker) = &cue.speaker {
            output = format!("<v {speaker}>{output}</v>");
        }
    }
    output
}

fn plain(value: &[char], format: CaptionFormat) -> String {
    let text: String = value.iter().collect();
    if format == CaptionFormat::Ass {
        text.replace('\n', "\\N")
    } else {
        text
    }
}

fn styled(
    mut text: String,
    style: &InlineStyle,
    format: CaptionFormat,
    cue: &CaptionCue,
    report: &mut LossReport,
) -> String {
    match format {
        CaptionFormat::Srt => {
            if let Some(color) = &style.color {
                text = format!("<font color=\"{color}\">{text}</font>");
            }
            if style.voice.is_some() {
                report.cue(
                    &cue.id,
                    "text.spans.voice",
                    "SRT cannot represent voice spans",
                );
            }
            html_emphasis(text, style)
        }
        CaptionFormat::WebVtt => {
            if style.color.is_some() {
                report.cue(
                    &cue.id,
                    "text.spans.color",
                    "WebVTT color requires an external style sheet",
                );
            }
            text = html_emphasis(text, style);
            style
                .voice
                .as_ref()
                .map_or(text.clone(), |voice| format!("<v {voice}>{text}</v>"))
        }
        CaptionFormat::Ass => {
            if style.voice.is_some() {
                report.cue(
                    &cue.id,
                    "text.spans.voice",
                    "ASS has only cue-level actor metadata",
                );
            }
            let mut tags = String::new();
            if style.bold {
                tags.push_str("\\b1");
            }
            if style.italic {
                tags.push_str("\\i1");
            }
            if style.underline {
                tags.push_str("\\u1");
            }
            if let Some(color) = &style.color {
                tags.push_str(&format!("\\c{color}"));
            }
            format!("{{{tags}}}{text}{{\\r}}")
        }
    }
}

fn html_emphasis(mut text: String, style: &InlineStyle) -> String {
    if style.underline {
        text = format!("<u>{text}</u>");
    }
    if style.italic {
        text = format!("<i>{text}</i>");
    }
    if style.bold {
        text = format!("<b>{text}</b>");
    }
    text
}
