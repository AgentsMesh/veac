use crate::{CaptionCue, ValidationIssue};

use crate::validation::{issue, optional_text};

pub(super) fn validate(cue: &CaptionCue, path: &str, issues: &mut Vec<ValidationIssue>) {
    let length = cue.text.plain.chars().count() as u32;
    let mut end = 0;
    for (index, span) in cue.text.spans.iter().enumerate() {
        let span_path = format!("{path}.text.spans[{index}]");
        if span.range.start >= span.range.end || span.range.end > length || span.range.start < end {
            issue(
                issues,
                &span_path,
                "TEXT_RANGE",
                "span must be ordered, nonempty, and contained",
            );
        }
        if span.style.is_plain() {
            issue(
                issues,
                &span_path,
                "EMPTY_STYLE",
                "span must apply at least one style",
            );
        }
        optional_text(
            issues,
            &format!("{span_path}.style.color"),
            span.style.color.as_deref(),
        );
        optional_text(
            issues,
            &format!("{span_path}.style.voice"),
            span.style.voice.as_deref(),
        );
        end = span.range.end;
    }
}
