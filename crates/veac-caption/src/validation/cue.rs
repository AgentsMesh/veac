mod text;
mod words;

use std::collections::BTreeSet;

use veac_ir::{RationalTime, TimeRange};

use crate::{CaptionCue, CaptionDocument, OverlapPolicy, ValidationIssue};

use super::{duplicates, issue, native, optional_text};

pub(super) fn validate_cues(document: &CaptionDocument, issues: &mut Vec<ValidationIssue>) {
    if duplicates(document.cues.iter().map(|cue| cue.id.as_str())) {
        issue(
            issues,
            "$.document.cues",
            "DUPLICATE_ID",
            "cue IDs must be unique",
        );
    }
    let style_ids: BTreeSet<_> = document
        .styles
        .iter()
        .map(|style| style.id.as_str())
        .collect();
    let mut previous: Option<&CaptionCue> = None;
    for (index, cue) in document.cues.iter().enumerate() {
        let path = format!("$.document.cues[{index}]");
        validate_cue(document, cue, &style_ids, &path, issues);
        if let Some(prior) = previous {
            if cue.range.start < prior.range.start {
                issue(issues, &path, "ORDER", "cues must be sorted by start time");
            }
            if document.overlap_policy == OverlapPolicy::Reject
                && ends_after(prior.range, cue.range.start)
            {
                issue(issues, &path, "OVERLAP", "cue overlaps the preceding cue");
            }
        }
        previous = Some(cue);
    }
}

fn validate_cue(
    document: &CaptionDocument,
    cue: &CaptionCue,
    styles: &BTreeSet<&str>,
    path: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if !cue.id.is_valid() {
        issue(issues, &format!("{path}.id"), "ID", "invalid cue ID");
    }
    validate_range(
        document.timescale,
        cue.range,
        &format!("{path}.range"),
        issues,
    );
    if cue.text.plain.trim().is_empty() {
        issue(
            issues,
            &format!("{path}.text.plain"),
            "EMPTY",
            "cue text must be nonempty",
        );
    }
    optional_text(issues, &format!("{path}.speaker"), cue.speaker.as_deref());
    optional_text(issues, &format!("{path}.style"), cue.style.as_deref());
    if cue
        .style
        .as_deref()
        .is_some_and(|style| !styles.contains(style))
    {
        issue(
            issues,
            &format!("{path}.style"),
            "STYLE_REF",
            "style reference does not exist",
        );
    }
    native::cue(cue.native.as_ref(), &format!("{path}.native"), issues);
    if !native::compatible(document.native.as_ref(), cue.native.as_ref()) {
        issue(
            issues,
            &format!("{path}.native"),
            "NATIVE_FORMAT",
            "cue native semantics conflict with the document native format",
        );
    }
    text::validate(cue, path, issues);
    words::validate(document.timescale, cue, path, issues);
}

pub(super) fn validate_range(
    timescale: u32,
    range: TimeRange,
    path: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if range.start.timescale != timescale || range.duration.timescale != timescale {
        issue(
            issues,
            path,
            "TIMESCALE",
            "range must use the document timescale",
        );
    }
    if !range.start.is_valid()
        || !range.duration.is_valid()
        || range.start.value < 0
        || range.duration.value <= 0
    {
        issue(
            issues,
            path,
            "TIME_RANGE",
            "range must be valid, nonnegative, and nonempty",
        );
    }
    if range.end().is_err() {
        issue(
            issues,
            path,
            "TIME_OVERFLOW",
            "range end overflows exact time",
        );
    }
}

fn ends_after(range: TimeRange, next: RationalTime) -> bool {
    range.end().is_ok_and(|end| end > next)
}
