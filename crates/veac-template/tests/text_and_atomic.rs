#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::*;
use veac_template::propose_template_fill;

#[test]
fn text_override_and_all_template_flags_clear_atomically() {
    let project = project(FillMode::TakeHead, true);
    let original_range = clip(&project, "itm_slot").record_range;
    let original_scale = clip(&project, "itm_slot")
        .visual
        .as_ref()
        .unwrap()
        .transform
        .scale
        .clone();
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.source = MaterialSource::File {
        uri: "media/published.mp4".to_owned(),
    };
    let mut request = request(&project, replacement);
    request.text_bindings.push(text_binding("Published title"));
    let batch = propose_template_fill(&project, &request).unwrap();
    let output = apply(&project, &batch);
    let media = clip(&output, "itm_slot");
    let title = clip(&output, "itm_title");
    assert_eq!(media.record_range, original_range);
    assert_eq!(
        media.visual.as_ref().unwrap().transform.scale,
        original_scale
    );
    assert_eq!(media.replaceable, None);
    assert_eq!(title.replaceable, None);
    assert!(!title.template_editable_text);
    assert!(matches!(&title.source, ClipSource::Text { text, .. } if text == "Published title"));
    assert!(project.project.materials[0].source != output.project.materials[0].source);
    assert_eq!(project.project.revision, 4);
    assert_eq!(output.project.revision, 5);
}

#[test]
fn omitted_optional_text_keeps_default_but_clears_the_flag() {
    let project = project(FillMode::FitDuration, true);
    let batch = propose_template_fill(
        &project,
        &request(&project, video(six_seconds(), 1080, 1920)),
    )
    .unwrap();
    let output = apply(&project, &batch);
    let title = clip(&output, "itm_title");
    assert!(matches!(&title.source, ClipSource::Text { text, .. } if text == "Default title"));
    assert_eq!(title.replaceable, None);
    assert!(!title.template_editable_text);
}

#[test]
fn proposal_contains_defensive_preconditions_for_every_target() {
    let project = project(FillMode::FitDuration, true);
    let batch = propose_template_fill(
        &project,
        &request(&project, video(six_seconds(), 1080, 1920)),
    )
    .unwrap();
    let exists = batch
        .preconditions
        .iter()
        .filter(|value| matches!(value, Precondition::ClipExists { .. }))
        .count();
    let source = batch
        .preconditions
        .iter()
        .filter(|value| matches!(value, Precondition::ClipSourceEquals { .. }))
        .count();
    let unlocked = batch
        .preconditions
        .iter()
        .filter(|value| matches!(value, Precondition::TrackUnlocked { .. }))
        .count();
    assert_eq!((exists, source, unlocked), (2, 2, 2));
}

#[test]
fn text_only_template_is_a_valid_atomic_proposal() {
    let mut project = project(FillMode::FitDuration, true);
    project.project.sequences[0].tracks[0].clips[0].replaceable = None;
    validate(&project).unwrap();
    let mut request = request(&project, video(six_seconds(), 1080, 1920));
    request.media_bindings.clear();
    request.text_bindings.push(text_binding("Text only"));
    let output = apply(
        &project,
        &propose_template_fill(&project, &request).unwrap(),
    );
    let title = clip(&output, "itm_title");
    assert!(matches!(&title.source, ClipSource::Text { text, .. } if text == "Text only"));
    assert_eq!(title.replaceable, None);
    assert!(!title.template_editable_text);
}

#[test]
fn locked_text_slot_is_not_a_fill_target() {
    let mut project = project(FillMode::FitDuration, true);
    project.project.sequences[0].tracks[0].clips[0].replaceable = None;
    project.project.sequences[0].tracks[1].clips[0].template_editable_text = false;
    validate(&project).unwrap();
    let mut request = request(&project, video(six_seconds(), 1080, 1920));
    request.media_bindings.clear();
    assert_eq!(
        error_kind(&project, &request),
        veac_template::TemplateErrorKind::NoTemplateTargets
    );
}
