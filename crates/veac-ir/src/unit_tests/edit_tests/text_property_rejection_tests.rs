use super::*;

#[test]
fn text_property_rejects_non_text_locks_and_invalid_final_state() {
    let project = sample_project();
    let video = ItemId::new("itm_video").unwrap();
    let edit = batch(
        "op_non_text_property",
        &project,
        vec![text(&video, TextProperty::SizePixels(30.0))],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");

    let caption = ItemId::new("itm_caption").unwrap();
    let invalid = batch(
        "op_invalid_text_property",
        &project,
        vec![text(&caption, TextProperty::LineHeight(0.0))],
    );
    assert_rejected(apply_edit_batch(&project, &invalid), "TEXT_STYLE");

    let mut locked = project;
    locked.project.sequences[0].tracks[1].state.locked = true;
    let edit = batch(
        "op_locked_text_property",
        &locked,
        vec![text(
            &caption,
            TextProperty::Color(Color {
                red: 1,
                green: 2,
                blue: 3,
                alpha: 255,
            }),
        )],
    );
    assert_rejected(apply_edit_batch(&locked, &edit), "EDIT_REJECTED");
}

fn text(id: &ItemId, property: TextProperty) -> EditOperation {
    EditOperation::SetTextProperty {
        clip_id: id.clone(),
        property,
    }
}
