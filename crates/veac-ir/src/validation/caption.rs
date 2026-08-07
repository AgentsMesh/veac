use crate::*;

use super::Validator;

const MAX_CAPTION_SPANS: usize = 4_096;
const MAX_CAPTION_WORDS: usize = 65_536;

impl Validator {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn caption_semantics(
        &mut self,
        value: &CaptionCueSemantics,
        text: &str,
        cue_range: TimeRange,
        timebase: u32,
        path: &str,
        object_id: &str,
    ) {
        if value.spans.len() > MAX_CAPTION_SPANS || value.words.len() > MAX_CAPTION_WORDS {
            self.value_error("CAPTION_SEMANTICS_BUDGET", path, object_id);
        }
        if value
            .style
            .as_ref()
            .is_some_and(|style| invalid_text(&style.0, 256) || style.0.trim().is_empty())
        {
            self.value_error("CAPTION_STYLE_REFERENCE", path, object_id);
        }
        self.caption_spans(&value.spans, text, path, object_id);
        self.caption_words(&value.words, cue_range, timebase, path, object_id);
        if value.native.as_ref().is_some_and(invalid_native) {
            self.value_error("CAPTION_NATIVE", path, object_id);
        }
    }

    fn caption_spans(
        &mut self,
        spans: &[CaptionMarkupSpan],
        text: &str,
        path: &str,
        object_id: &str,
    ) {
        let length = text.chars().count() as u32;
        let mut end = 0;
        for span in spans {
            let style = &span.style;
            let empty = !style.bold
                && !style.italic
                && !style.underline
                && style.color.is_none()
                && style.voice.is_none();
            let invalid = span.range.start >= span.range.end
                || span.range.start < end
                || span.range.end > length
                || empty
                || style
                    .color
                    .as_ref()
                    .is_some_and(|text| invalid_text(text, 256))
                || style
                    .voice
                    .as_ref()
                    .is_some_and(|text| invalid_text(text, 256));
            if invalid {
                self.value_error("CAPTION_SPAN", path, object_id);
            }
            end = span.range.end;
        }
    }

    fn caption_words(
        &mut self,
        words: &[CaptionWordTiming],
        cue: TimeRange,
        timebase: u32,
        path: &str,
        object_id: &str,
    ) {
        let cue_end = cue.end().ok();
        let mut previous = cue.start;
        for word in words {
            self.time_range(word.range, timebase, "CAPTION_WORD", path, object_id);
            let invalid = word.text.trim().is_empty()
                || word.text.len() > MAX_TEXT_BYTES
                || word.range.start < cue.start
                || word.range.start < previous
                || cue_end.is_some_and(|end| word.range.end().is_ok_and(|value| value > end))
                || word
                    .confidence
                    .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value));
            if invalid {
                self.value_error("CAPTION_WORD", path, object_id);
            }
            previous = word.range.end().unwrap_or(previous);
        }
    }
}

fn invalid_webvtt(value: &WebVttCueSettings) -> bool {
    value
        .line
        .as_ref()
        .is_some_and(|value| invalid_token(value))
        || value
            .position
            .as_ref()
            .is_some_and(|value| !percent_token(value))
        || value
            .size
            .as_ref()
            .is_some_and(|value| !percent_token(value))
        || value.region.as_ref().is_some_and(|value| {
            invalid_text(&value.0, 256) || value.0.contains(char::is_whitespace)
        })
}

fn invalid_native(value: &CaptionNativeCue) -> bool {
    match value {
        CaptionNativeCue::Srt { index } => *index == 0 || *index > MAX_SAFE_INTEGER,
        CaptionNativeCue::WebVtt {
            identifier,
            settings,
        } => {
            identifier
                .as_ref()
                .is_some_and(|value| invalid_text(&value.0, 256))
                || settings.as_ref().is_some_and(invalid_webvtt)
        }
        CaptionNativeCue::Ass { settings } => settings
            .effect
            .as_ref()
            .is_some_and(|value| invalid_text(value, 1_024) || value.contains(',')),
    }
}

fn percent_token(value: &str) -> bool {
    value
        .strip_suffix('%')
        .and_then(|number| number.parse::<f64>().ok())
        .is_some_and(|number| number.is_finite() && (0.0..=100.0).contains(&number))
}

fn invalid_token(value: &str) -> bool {
    invalid_text(value, 128) || value.contains(char::is_whitespace)
}

fn invalid_text(value: &str, maximum: usize) -> bool {
    value.is_empty() || value.len() > maximum || value.chars().any(char::is_control)
}
