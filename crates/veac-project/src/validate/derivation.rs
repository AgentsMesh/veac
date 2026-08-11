use crate::{
    IssueCode, MediaDerivation, ProjectInputSource, ProjectIssue, ProjectOutput, ProjectTarget,
};

mod parameters;

use parameters::*;

pub(super) fn check(
    target: &ProjectTarget,
    operation: &MediaDerivation,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    source(target, operation, base, issues);
    output(target, operation, base, issues);
    parameters(operation, base, issues);
}

fn source(
    target: &ProjectTarget,
    operation: &MediaDerivation,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let Some(input) = target
        .inputs
        .iter()
        .find(|input| input.id == *operation.source())
    else {
        return issue(
            issues,
            IssueCode::UnknownReference,
            &format!("{base}.entry.operation.source"),
            format!("unknown target input {:?}", operation.source().as_str()),
        );
    };
    if !matches!(
        input.source,
        ProjectInputSource::ProjectMaterial { .. } | ProjectInputSource::Artifact { .. }
    ) {
        issue(
            issues,
            IssueCode::InvalidAction,
            &format!("{base}.entry.operation.source"),
            "media derivation source must be project material or one artifact",
        );
    }
}

fn output(
    target: &ProjectTarget,
    operation: &MediaDerivation,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let expected = operation.output_media_type();
    let valid = matches!(
        target.outputs.as_slice(),
        [ProjectOutput::Media { media_type, .. }] if *media_type == expected
    );
    if !valid {
        issue(
            issues,
            IssueCode::InvalidAction,
            &format!("{base}.outputs"),
            format!("media derivation requires exactly one {expected:?} media output"),
        );
    }
}

fn parameters(operation: &MediaDerivation, base: &str, issues: &mut Vec<ProjectIssue>) {
    let path = format!("{base}.entry.operation");
    match operation {
        MediaDerivation::ProxyVideo {
            source_clock,
            width,
            height,
            frame_rate,
            crf,
            ..
        } => {
            clock(*source_clock, &path, issues);
            dimensions(*width, *height, true, &path, issues);
            rate(*frame_rate, &format!("{path}.frame_rate"), issues);
            crf_value(*crf, &format!("{path}.crf"), issues);
        }
        MediaDerivation::ProxyAudio {
            source_clock,
            sample_rate,
            channels,
            ..
        } => {
            clock(*source_clock, &path, issues);
            audio(*sample_rate, *channels, &path, issues);
        }
        MediaDerivation::Thumbnail {
            at, width, height, ..
        } => {
            time(*at, false, &format!("{path}.at"), issues);
            dimensions(*width, *height, false, &path, issues);
        }
        MediaDerivation::Waveform {
            source_clock,
            sample_rate,
            width,
            height,
            color,
            ..
        } => {
            clock(*source_clock, &path, issues);
            audio(*sample_rate, 1, &path, issues);
            dimensions(*width, *height, false, &path, issues);
            if color.is_empty() || !color.bytes().all(valid_color) {
                issue(
                    issues,
                    IssueCode::InvalidAction,
                    &format!("{path}.color"),
                    "waveform color contains unsupported characters",
                );
            }
        }
        MediaDerivation::OpticalFlow {
            source_clock,
            width,
            height,
            frame_rate,
            ..
        } => {
            clock(*source_clock, &path, issues);
            dimensions(*width, *height, true, &path, issues);
            if *width < 32 || *height < 32 {
                issue(
                    issues,
                    IssueCode::InvalidAction,
                    &path,
                    "optical-flow dimensions must be at least 32x32",
                );
            }
            rate(*frame_rate, &format!("{path}.frame_rate"), issues);
        }
        MediaDerivation::SourceSegment {
            start,
            duration,
            width,
            height,
            frame_rate,
            audio: settings,
            crf,
            ..
        } => {
            time(*start, false, &format!("{path}.start"), issues);
            time(*duration, true, &format!("{path}.duration"), issues);
            common_timescale(*start, *duration, &path, issues);
            dimensions(*width, *height, true, &path, issues);
            rate(*frame_rate, &format!("{path}.frame_rate"), issues);
            if let Some(settings) = settings {
                segment_audio(*settings, &path, issues);
            }
            crf_value(*crf, &format!("{path}.crf"), issues);
        }
    }
}

fn issue(issues: &mut Vec<ProjectIssue>, code: IssueCode, path: &str, message: impl Into<String>) {
    issues.push(ProjectIssue::new(code, path, message));
}
