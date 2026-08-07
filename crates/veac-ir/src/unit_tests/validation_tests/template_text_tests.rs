use super::*;
use crate::test_support::time;

fn slot() -> SlotConstraint {
    SlotConstraint {
        kind: SlotKind::Text,
        fill: FillMode::FitDuration,
        label: "Title".to_owned(),
        min_source_duration: None,
    }
}

fn media_slot() -> SlotConstraint {
    SlotConstraint {
        kind: SlotKind::Video,
        fill: FillMode::TakeHead,
        label: "Hero".to_owned(),
        min_source_duration: None,
    }
}

fn text_clip(project: &mut ProjectEnvelope) -> &mut Clip {
    let track = &mut project.project.sequences[0].tracks[1];
    track.kind = TrackKind::Visual;
    let clip = &mut track.clips[0];
    let ClipSource::Caption { text, style, .. } = &clip.source else {
        panic!("caption fixture");
    };
    clip.source = ClipSource::Text {
        text: text.clone(),
        style: style.clone(),
    };
    clip
}

#[test]
fn accepts_locked_and_editable_text_slots() {
    for editable in [false, true] {
        let mut project = sample_project();
        let clip = text_clip(&mut project);
        clip.replaceable = Some(slot());
        clip.template_editable_text = editable;
        validate(&project).unwrap();
    }
}

#[test]
fn text_slots_reject_every_non_text_source_family() {
    let caption = sample_project().project.sequences[0].tracks[1].clips[0]
        .source
        .clone();
    let sources = [
        ClipSource::Media {
            material_id: MaterialId::new("med_video").unwrap(),
        },
        caption,
        ClipSource::Generated {
            generator: Generator::Transparent,
        },
    ];
    for source in sources {
        let mut project = sample_project();
        let track = &mut project.project.sequences[0].tracks[1];
        track.kind = TrackKind::Visual;
        track.clips[0].source = source;
        track.clips[0].replaceable = Some(slot());
        assert_code(&validation_codes(&project), "TEMPLATE_SLOT_SOURCE");
    }
}

#[test]
fn media_slots_reject_text_sources_and_editable_text_policy() {
    let mut project = sample_project();
    let clip = text_clip(&mut project);
    clip.replaceable = Some(media_slot());
    clip.template_editable_text = true;
    let codes = validation_codes(&project);
    assert_code(&codes, "TEMPLATE_SLOT_SOURCE");
    assert_code(&codes, "TEMPLATE_EDITABLE_TEXT");
}

#[test]
fn editable_text_requires_a_typed_text_slot_and_text_source() {
    let mut missing = sample_project();
    text_clip(&mut missing).template_editable_text = true;
    assert_code(&validation_codes(&missing), "TEMPLATE_EDITABLE_TEXT");

    let mut caption = sample_project();
    let clip = &mut caption.project.sequences[0].tracks[1].clips[0];
    clip.replaceable = Some(slot());
    clip.template_editable_text = true;
    assert_code(&validation_codes(&caption), "TEMPLATE_EDITABLE_TEXT");
}

#[test]
fn text_slots_reject_media_timing_policies() {
    let mut project = sample_project();
    let clip = text_clip(&mut project);
    let mut constraint = slot();
    constraint.fill = FillMode::TakeCenter;
    constraint.min_source_duration = Some(time(600));
    clip.replaceable = Some(constraint);
    let codes = validation_codes(&project);
    assert_code(&codes, "TEMPLATE_SLOT_TEXT_FILL");
    assert_code(&codes, "TEMPLATE_SLOT_MIN_KIND");
}
