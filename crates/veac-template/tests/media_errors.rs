#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::*;
use veac_template::TemplateErrorKind;

#[test]
fn material_identity_and_kind_are_slot_contracts() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.id = MaterialId::new("med_other").unwrap();
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MaterialIdMismatch
    );

    let mut project = support::project(FillMode::FitDuration, false);
    clip_mut(&mut project).replaceable.as_mut().unwrap().kind = SlotKind::Video;
    validate(&project).unwrap();
    assert_eq!(
        error_kind(&project, &request(&project, image(1000, 1000))),
        TemplateErrorKind::KindMismatch
    );
}

#[test]
fn probe_and_selected_stream_facts_are_mandatory() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe = None;
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingProbe
    );

    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe.as_mut().unwrap().selected_video_stream = None;
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingProbeFacts
    );

    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe.as_mut().unwrap().selected_video_stream = Some(StreamSelection {
        global_index: 9,
        type_index: 0,
    });
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingProbeFacts
    );

    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe.as_mut().unwrap().streams[0].video = None;
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingProbeFacts
    );

    let replacement = video(six_seconds(), 0, 1080);
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingProbeFacts
    );

    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe.as_mut().unwrap().streams[0].duration = None;
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingDuration
    );
}

#[test]
fn every_video_fill_respects_an_explicit_minimum_duration() {
    for fill in [
        FillMode::FitDuration,
        FillMode::TakeHead,
        FillMode::TakeCenter,
    ] {
        let mut project = project(fill, false);
        let constraint = clip_mut(&mut project).replaceable.as_mut().unwrap();
        constraint.kind = SlotKind::Video;
        constraint.min_source_duration = Some(time(3000));
        validate(&project).unwrap();
        let replacement = video(time(2700), 1920, 1080);
        assert_eq!(
            error_kind(&project, &request(&project, replacement)),
            TemplateErrorKind::MediaTooShort
        );
    }
}

#[test]
fn foreign_and_half_tick_source_times_are_rejected() {
    let project = project(FillMode::FitDuration, false);
    let replacement = video(RationalTime::new(1, 1001).unwrap(), 1920, 1080);
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::InexactTime
    );

    let replacement = video(time(0), 1920, 1080);
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingProbeFacts
    );

    let project = support::project(FillMode::TakeCenter, false);
    let replacement = video(time(3001), 1920, 1080);
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::InexactTime
    );
}

#[test]
fn invalid_overflowed_and_unrepresentable_times_fail_closed() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe.as_mut().unwrap().streams[0].duration = Some(RationalTime {
        value: 1,
        timescale: 0,
    });
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::InexactTime
    );

    let project = support::project(FillMode::FitDuration, false);
    let huge = RationalTime::new(veac_ir::MAX_SAFE_INTEGER as i64, 1).unwrap();
    assert_eq!(
        error_kind(&project, &request(&project, video(huge, 1920, 1080))),
        TemplateErrorKind::InexactTime
    );
}

fn clip_mut(project: &mut ProjectEnvelope) -> &mut Clip {
    &mut project.project.sequences[0].tracks[0].clips[0]
}
