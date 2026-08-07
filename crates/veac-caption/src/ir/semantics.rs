use veac_ir::{
    CaptionCueSemantics, CaptionInlineStyle, CaptionMarkupSpan, CaptionStyleReference,
    CaptionTextRange, CaptionWordTiming, TimeRange,
};

use crate::{
    CaptionCue, CaptionCueId, CaptionError, CaptionSpan, CaptionText, CaptionWord, InlineStyle,
    TextRange,
};

pub(crate) mod webvtt;

pub(super) fn encode(cue: &CaptionCue) -> Result<CaptionCueSemantics, CaptionError> {
    Ok(CaptionCueSemantics {
        spans: cue.text.spans.iter().map(encode_span).collect(),
        style: cue.style.clone().map(CaptionStyleReference),
        native: cue.native.clone(),
        words: cue.words.iter().map(encode_word).collect(),
    })
}

pub(super) fn decode(
    id: CaptionCueId,
    range: TimeRange,
    text: &str,
    speaker: Option<&str>,
    semantics: &CaptionCueSemantics,
) -> Result<CaptionCue, CaptionError> {
    Ok(CaptionCue {
        id,
        range,
        text: CaptionText {
            plain: text.to_owned(),
            spans: semantics.spans.iter().map(decode_span).collect(),
        },
        speaker: speaker.map(str::to_owned),
        style: semantics.style.as_ref().map(|value| value.0.clone()),
        native: semantics.native.clone(),
        words: semantics.words.iter().map(decode_word).collect(),
    })
}

fn encode_span(value: &CaptionSpan) -> CaptionMarkupSpan {
    CaptionMarkupSpan {
        range: CaptionTextRange {
            start: value.range.start,
            end: value.range.end,
        },
        style: CaptionInlineStyle {
            bold: value.style.bold,
            italic: value.style.italic,
            underline: value.style.underline,
            color: value.style.color.clone(),
            voice: value.style.voice.clone(),
        },
    }
}

fn decode_span(value: &CaptionMarkupSpan) -> CaptionSpan {
    CaptionSpan {
        range: TextRange {
            start: value.range.start,
            end: value.range.end,
        },
        style: InlineStyle {
            bold: value.style.bold,
            italic: value.style.italic,
            underline: value.style.underline,
            color: value.style.color.clone(),
            voice: value.style.voice.clone(),
        },
    }
}

fn encode_word(value: &CaptionWord) -> CaptionWordTiming {
    CaptionWordTiming {
        text: value.text.clone(),
        range: value.range,
        confidence: value.confidence,
    }
}

fn decode_word(value: &CaptionWordTiming) -> CaptionWord {
    CaptionWord {
        text: value.text.clone(),
        range: value.range,
        confidence: value.confidence,
    }
}
