#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::*;
use veac_template::{propose_template_fill, TemplateErrorKind};

#[test]
fn request_header_revision_and_template_presence_are_strict() {
    let project = project(FillMode::FitDuration, false);
    let mut value = request(&project, video(six_seconds(), 1920, 1080));
    value.schema_version = 2;
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::InvalidRequest
    );

    let mut value = request(&project, video(six_seconds(), 1920, 1080));
    value.base_revision = veac_ir::MAX_SAFE_INTEGER + 1;
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::InvalidRequest
    );

    let mut value = request(&project, video(six_seconds(), 1920, 1080));
    value.base_revision += 1;
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::RevisionMismatch
    );

    let mut ordinary = project.clone();
    ordinary.project.sequences[0].tracks[0].clips[0].replaceable = None;
    validate(&ordinary).unwrap();
    let value = request(&ordinary, video(six_seconds(), 1920, 1080));
    assert_eq!(
        error_kind(&ordinary, &value),
        TemplateErrorKind::NoTemplateTargets
    );
}

#[test]
fn invalid_project_and_locked_targets_fail_before_proposal() {
    let mut invalid = project(FillMode::FitDuration, false);
    invalid.project.sequences[0].tracks[0].clips[0]
        .replaceable
        .as_mut()
        .unwrap()
        .label = "".to_owned();
    let value = request(&invalid, video(six_seconds(), 1920, 1080));
    assert_eq!(
        error_kind(&invalid, &value),
        TemplateErrorKind::InvalidProject
    );

    let mut locked = project(FillMode::FitDuration, true);
    locked.project.sequences[0].tracks[1].state.locked = true;
    let value = request(&locked, video(six_seconds(), 1920, 1080));
    assert_eq!(error_kind(&locked, &value), TemplateErrorKind::LockedTrack);
}

#[test]
fn invalid_replacement_is_rejected_by_the_atomic_dry_run() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.source = MaterialSource::File { uri: "".to_owned() };
    let error = propose_template_fill(&project, &request(&project, replacement)).unwrap_err();
    assert_eq!(error.kind, TemplateErrorKind::EditRejected);
    assert!(error.message.contains("rejected"));
}
