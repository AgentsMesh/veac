use crate::{IssueCode, ProjectIssue, ProjectRational, ProjectSegmentAudio, ProjectSourceClock};

use super::issue;

pub(super) fn clock(value: ProjectSourceClock, path: &str, issues: &mut Vec<ProjectIssue>) {
    match value {
        ProjectSourceClock::Identity { duration } => {
            time(
                duration,
                true,
                &format!("{path}.source_clock.duration"),
                issues,
            );
        }
        ProjectSourceClock::Bounded { start, duration } => {
            time(start, false, &format!("{path}.source_clock.start"), issues);
            time(
                duration,
                true,
                &format!("{path}.source_clock.duration"),
                issues,
            );
            common_timescale(start, duration, path, issues);
        }
    }
}

pub(super) fn common_timescale(
    start: ProjectRational,
    duration: ProjectRational,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    if start.denominator == 0 || duration.denominator == 0 {
        return;
    }
    let divisor = gcd(start.denominator, duration.denominator);
    let Some(timescale) = start
        .denominator
        .checked_div(divisor)
        .and_then(|value| value.checked_mul(duration.denominator))
    else {
        return issue(
            issues,
            IssueCode::InvalidAction,
            path,
            "source clock timescale overflow",
        );
    };
    let safe = |value: ProjectRational| {
        let scaled = i128::from(value.numerator) * i128::from(timescale / value.denominator);
        scaled.unsigned_abs() <= 9_007_199_254_740_991_u128
    };
    if timescale > u64::from(u32::MAX) || !safe(start) || !safe(duration) {
        issue(
            issues,
            IssueCode::InvalidAction,
            path,
            "source clock cannot share an exact u32 timescale",
        );
    }
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

pub(super) fn dimensions(
    width: u32,
    height: u32,
    encoded_video: bool,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    if width == 0 || height == 0 || (encoded_video && (width % 2 != 0 || height % 2 != 0)) {
        issue(
            issues,
            IssueCode::InvalidAction,
            path,
            "media dimensions are invalid for the output format",
        );
    }
}

pub(super) fn audio(sample_rate: u32, channels: u8, path: &str, issues: &mut Vec<ProjectIssue>) {
    if sample_rate == 0 || !(1..=8).contains(&channels) {
        issue(
            issues,
            IssueCode::InvalidAction,
            path,
            "audio sample rate and channel count are invalid",
        );
    }
}

pub(super) fn segment_audio(
    value: ProjectSegmentAudio,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    audio(
        value.sample_rate,
        value.channels,
        &format!("{path}.audio"),
        issues,
    );
}

pub(super) fn rate(value: ProjectRational, path: &str, issues: &mut Vec<ProjectIssue>) {
    rational(value, true, false, path, issues);
}

pub(super) fn time(
    value: ProjectRational,
    positive: bool,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    rational(value, positive, true, path, issues);
}

fn rational(
    value: ProjectRational,
    positive: bool,
    microseconds: bool,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let sign = if positive {
        value.numerator > 0
    } else {
        value.numerator >= 0
    };
    let exact = !microseconds
        || value.denominator != 0
            && i128::from(value.numerator) * 1_000_000 % i128::from(value.denominator) == 0;
    if !value.is_canonical() || value.denominator > u64::from(u32::MAX) || !sign || !exact {
        issue(
            issues,
            IssueCode::InvalidAction,
            path,
            "rational must be reduced, fit u32 timescale, and use exact microseconds",
        );
    }
}

pub(super) fn crf_value(value: u8, path: &str, issues: &mut Vec<ProjectIssue>) {
    if value > 51 {
        issue(
            issues,
            IssueCode::InvalidAction,
            path,
            "CRF must be at most 51",
        );
    }
}

pub(super) fn valid_color(value: u8) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, b'#' | b'@' | b'_' | b'-')
}
