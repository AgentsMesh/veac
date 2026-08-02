use subtitler::model::{Subtitle, TextPart};

use crate::{CaptionError, CaptionSpan, CaptionText, InlineStyle, TextRange};

pub(super) fn from_parts(
    plain: &str,
    parts: &[TextPart],
) -> Result<(CaptionText, Option<String>), CaptionError> {
    let mut byte_cursor = 0;
    let mut spans = Vec::new();
    for part in parts {
        let offset = plain[byte_cursor..].find(&part.text).ok_or_else(|| {
            CaptionError::time("subtitle parser returned a rich span outside plain text")
        })?;
        let start_byte = byte_cursor + offset;
        let end_byte = start_byte + part.text.len();
        let style = InlineStyle {
            bold: part.bold(),
            italic: part.italic(),
            underline: part.underline(),
            color: part.color.clone(),
            voice: part.voice.clone(),
        };
        if !style.is_plain() {
            spans.push(CaptionSpan {
                range: TextRange {
                    start: plain[..start_byte].chars().count() as u32,
                    end: plain[..end_byte].chars().count() as u32,
                },
                style,
            });
        }
        byte_cursor = end_byte;
    }
    let speaker = full_voice(plain, &mut spans);
    Ok((
        CaptionText {
            plain: plain.to_owned(),
            spans,
        },
        speaker,
    ))
}

pub(super) fn from_ass(subtitle: &Subtitle) -> Result<CaptionText, CaptionError> {
    let plain = subtitler::ass::ass_to_plaintext(&subtitle.text);
    let parts = subtitler::ass::parse_ass_tags(&subtitle.text);
    from_parts(&plain, &parts).map(|value| value.0)
}

fn full_voice(plain: &str, spans: &mut Vec<CaptionSpan>) -> Option<String> {
    let length = plain.chars().count() as u32;
    let voice = spans.first()?.style.voice.clone()?;
    let mut end = 0;
    for span in spans.iter() {
        if span.range.start != end || span.style.voice.as_deref() != Some(voice.as_str()) {
            return None;
        }
        end = span.range.end;
    }
    if end != length {
        return None;
    }
    spans.iter_mut().for_each(|span| span.style.voice = None);
    spans.retain(|span| !span.style.is_plain());
    Some(voice)
}
