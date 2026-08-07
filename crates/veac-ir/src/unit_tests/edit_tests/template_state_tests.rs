use super::*;

fn state(clip_id: ItemId, replaceable: Option<SlotConstraint>, text: bool) -> EditOperation {
    EditOperation::SetTemplateState {
        clip_id,
        replaceable,
        template_editable_text: text,
    }
}

fn slot() -> SlotConstraint {
    SlotConstraint {
        kind: SlotKind::Video,
        fill: FillMode::FitDuration,
        label: "Hero".to_owned(),
        min_source_duration: None,
    }
}

fn text_slot() -> SlotConstraint {
    SlotConstraint {
        kind: SlotKind::Text,
        fill: FillMode::FitDuration,
        label: "Title".to_owned(),
        min_source_duration: None,
    }
}

#[test]
fn clears_template_state_atomically() {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.replaceable = Some(slot());
    let id = clip.id.clone();
    let outcome = apply_edit_batch(
        &project,
        &batch("op_clear_template", &project, vec![state(id, None, false)]),
    );
    let output = applied(outcome);
    let clip = &output.project.sequences[0].tracks[0].clips[0];
    assert_eq!(clip.replaceable, None);
    assert!(!clip.template_editable_text);
}

#[test]
fn rejects_template_state_on_a_locked_track() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].state.locked = true;
    let id = project.project.sequences[0].tracks[0].clips[0].id.clone();
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_locked_template",
            &project,
            vec![state(id, Some(slot()), false)],
        ),
    );
    assert_rejected(outcome, "EDIT_REJECTED");
}

#[test]
fn post_validation_rejects_invalid_template_state() {
    let project = sample_project();
    let caption = &project.project.sequences[0].tracks[1].clips[0];
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_invalid_template",
            &project,
            vec![state(caption.id.clone(), None, true)],
        ),
    );
    assert_rejected(outcome, "TEMPLATE_EDITABLE_TEXT");
    assert!(!project.project.sequences[0].tracks[1].clips[0].template_editable_text);
}

#[test]
fn post_validation_accepts_a_typed_editable_text_state() {
    let mut project = sample_project();
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
    let id = clip.id.clone();
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_text_template",
            &project,
            vec![state(id, Some(text_slot()), true)],
        ),
    );
    let output = applied(outcome);
    let clip = &output.project.sequences[0].tracks[1].clips[0];
    assert_eq!(clip.replaceable.as_ref().unwrap().kind, SlotKind::Text);
    assert!(clip.template_editable_text);
}
