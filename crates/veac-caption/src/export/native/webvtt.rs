use std::fmt::Write;

use subtitler::model::Subtitle;
use veac_ir::CaptionNativeCue;

use crate::{CaptionDocument, CaptionDocumentNative};

use super::millis_timestamp;

pub(crate) fn render(document: &CaptionDocument, subtitles: &[Subtitle]) -> String {
    let mut output = header(document);
    output.push_str("\n\n");
    for (position, (subtitle, cue)) in subtitles.iter().zip(&document.cues).enumerate() {
        if let Some(CaptionNativeCue::WebVtt {
            identifier: Some(identifier),
            ..
        }) = &cue.native
        {
            let _ = writeln!(output, "{}", identifier.0);
        }
        let _ = write!(
            output,
            "{} --> {}",
            millis_timestamp(subtitle.start, '.'),
            millis_timestamp(subtitle.end, '.')
        );
        if let Some(settings) = &subtitle.settings {
            let _ = write!(output, " {settings}");
        }
        let _ = writeln!(output, "\n{}", subtitle.text);
        if position + 1 < subtitles.len() {
            output.push('\n');
        }
    }
    output
}

fn header(document: &CaptionDocument) -> String {
    let Some(CaptionDocumentNative::WebVtt { header }) = &document.native else {
        return "WEBVTT".to_owned();
    };
    match header.description.as_deref() {
        Some(description) => format!("WEBVTT {description}"),
        None => "WEBVTT".to_owned(),
    }
}
