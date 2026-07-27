use crate::{CaptionCue, ValidationIssue};

use crate::validation::issue;

use super::validate_range;

pub(super) fn validate(
    timescale: u32,
    cue: &CaptionCue,
    path: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cue_end = cue.range.end().ok();
    let mut prior_end = cue.range.start;
    for (index, word) in cue.words.iter().enumerate() {
        let word_path = format!("{path}.words[{index}]");
        validate_range(timescale, word.range, &format!("{word_path}.range"), issues);
        if word.text.trim().is_empty() {
            issue(
                issues,
                &format!("{word_path}.text"),
                "EMPTY",
                "word text must be nonempty",
            );
        }
        if word.range.start < cue.range.start
            || cue_end.is_some_and(|end| word.range.end().is_ok_and(|value| value > end))
        {
            issue(
                issues,
                &word_path,
                "WORD_CONTAINMENT",
                "word must be contained by its cue",
            );
        }
        if word.range.start < prior_end {
            issue(
                issues,
                &word_path,
                "WORD_ORDER",
                "words must be ordered and nonoverlapping",
            );
        }
        if word
            .confidence
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            issue(
                issues,
                &format!("{word_path}.confidence"),
                "CONFIDENCE",
                "must be finite in [0, 1]",
            );
        }
        prior_end = word.range.end().unwrap_or(prior_end);
    }
}
