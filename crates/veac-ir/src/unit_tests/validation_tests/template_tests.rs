use super::*;
use crate::test_support::{multicam_project, range, time};

fn slot(kind: SlotKind) -> SlotConstraint {
    SlotConstraint {
        kind,
        fill: FillMode::TakeHead,
        label: "Hero".to_owned(),
        min_source_duration: Some(time(600)),
    }
}

fn media(project: &mut ProjectEnvelope) -> &mut Clip {
    &mut project.project.sequences[0].tracks[0].clips[0]
}

#[test]
fn accepts_a_valid_slot_and_editable_text() {
    let mut project = sample_project();
    media(&mut project).replaceable = Some(slot(SlotKind::Video));
    let text_track = &mut project.project.sequences[0].tracks[1];
    text_track.kind = TrackKind::Visual;
    let clip = &mut text_track.clips[0];
    let ClipSource::Caption { text, style, .. } = &clip.source else {
        panic!("caption fixture");
    };
    clip.source = ClipSource::Text {
        text: text.clone(),
        style: style.clone(),
    };
    clip.template_editable_text = true;
    validate(&project).unwrap();
}

#[test]
fn rejects_slot_on_non_media_and_editable_caption() {
    let mut project = sample_project();
    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    caption.replaceable = Some(slot(SlotKind::VideoOrImage));
    caption.template_editable_text = true;
    let codes = validation_codes(&project);
    assert_code(&codes, "TEMPLATE_SLOT_SOURCE");
    assert_code(&codes, "TEMPLATE_EDITABLE_TEXT");
}

#[test]
fn rejects_placeholder_kind_and_shared_ownership() {
    let mut project = sample_project();
    media(&mut project).replaceable = Some(slot(SlotKind::Image));
    let mut shared = media(&mut project).clone();
    shared.id = ItemId::new("itm_shared").unwrap();
    shared.record_range = range(600, 600);
    project.project.sequences[0].tracks[0].clips.push(shared);
    let codes = validation_codes(&project);
    assert_code(&codes, "TEMPLATE_SLOT_KIND");
    assert_code(&codes, "TEMPLATE_SLOT_OWNERSHIP");
}

#[test]
fn rejects_empty_or_oversized_labels() {
    let mut project = sample_project();
    media(&mut project).replaceable = Some(slot(SlotKind::Video));
    media(&mut project).replaceable.as_mut().unwrap().label = " ".to_owned();
    assert_code(&validation_codes(&project), "TEMPLATE_SLOT_LABEL");
    media(&mut project).replaceable.as_mut().unwrap().label = "x".repeat(SLOT_LABEL_MAX_BYTES + 1);
    assert_code(&validation_codes(&project), "TEMPLATE_SLOT_LABEL");
}

#[test]
fn rejects_nonpositive_or_foreign_timebase_minimum() {
    let mut project = sample_project();
    media(&mut project).replaceable = Some(slot(SlotKind::Video));
    media(&mut project)
        .replaceable
        .as_mut()
        .unwrap()
        .min_source_duration = Some(RationalTime::new(0, 600).unwrap());
    assert_code(&validation_codes(&project), "TEMPLATE_SLOT_MIN_DURATION");
    media(&mut project)
        .replaceable
        .as_mut()
        .unwrap()
        .min_source_duration = Some(RationalTime::new(1, 1).unwrap());
    assert_code(&validation_codes(&project), "TEMPLATE_SLOT_MIN_DURATION");
}

#[test]
fn freeze_and_multicam_references_break_placeholder_ownership() {
    let mut freeze = sample_project();
    media(&mut freeze).replaceable = Some(slot(SlotKind::Video));
    let mut shared = media(&mut freeze).clone();
    shared.id = ItemId::new("itm_freeze").unwrap();
    shared.record_range = range(600, 600);
    shared.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new("med_video").unwrap(),
        source_time: time(0),
    };
    shared.source_mapping = None;
    shared.replaceable = None;
    freeze.project.sequences[0].tracks[0].clips.push(shared);
    assert_code(&validation_codes(&freeze), "TEMPLATE_SLOT_OWNERSHIP");

    let mut multicam = multicam_project();
    let mut direct = sample_project().project.sequences[0].tracks[0].clips[0].clone();
    direct.id = ItemId::new("itm_slot").unwrap();
    direct.record_range = range(600, 600);
    direct.replaceable = Some(slot(SlotKind::Video));
    multicam.project.sequences[0].tracks[0].clips.push(direct);
    assert_code(&validation_codes(&multicam), "TEMPLATE_SLOT_OWNERSHIP");
}

#[test]
fn slots_require_visual_tracks_and_authored_visual_state() {
    let mut audio = sample_project();
    let track = &mut audio.project.sequences[0].tracks[0];
    track.kind = TrackKind::Audio;
    let clip = &mut track.clips[0];
    clip.visual = None;
    clip.replaceable = Some(slot(SlotKind::Video));
    let codes = validation_codes(&audio);
    assert_code(&codes, "TEMPLATE_SLOT_TRACK");
    assert_code(&codes, "TEMPLATE_SLOT_VISUAL");

    let mut missing_visual = sample_project();
    media(&mut missing_visual).visual = None;
    media(&mut missing_visual).replaceable = Some(slot(SlotKind::Video));
    assert_code(&validation_codes(&missing_visual), "TEMPLATE_SLOT_VISUAL");
}

#[test]
fn minimum_duration_is_only_valid_for_video_slots() {
    for kind in [SlotKind::Image, SlotKind::VideoOrImage] {
        let mut project = sample_project();
        media(&mut project).replaceable = Some(slot(kind));
        assert_code(&validation_codes(&project), "TEMPLATE_SLOT_MIN_KIND");
    }
}
