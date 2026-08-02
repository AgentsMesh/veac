use super::*;

use super::apply_edit_lifecycle_tests::{apply, matte};

#[test]
fn locked_target_or_matte_producer_rolls_back_apply_edits() {
    let mut locked_target = sample_project();
    locked_target.project.sequences[0].tracks[0].state.locked = true;
    let insert = batch(
        "op_apply_locked_target",
        &locked_target,
        vec![structure(StructureEdit::InsertApply {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            apply: Box::new(apply(
                "apl_locked",
                "aps_locked",
                ApplyTarget::Layer {
                    track_id: TrackId::new("trk_video").unwrap(),
                },
            )),
            before_id: None,
            after_id: None,
        })],
    );
    assert_rejected(apply_edit_batch(&locked_target, &insert), "EDIT_REJECTED");

    let mut locked_source = sample_project();
    let mut apply = apply(
        "apl_matte_lock",
        "aps_matte_lock",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    apply.record_range = locked_source.project.sequences[0].tracks[1].clips[0].record_range;
    locked_source.project.sequences[0]
        .applies
        .push(apply.clone());
    locked_source.project.relations.push(matte(&apply.id));
    locked_source.project.sequences[0].tracks[1].state.locked = true;
    let before = locked_source.clone();
    let remove = batch(
        "op_apply_locked_source",
        &locked_source,
        vec![structure(StructureEdit::RemoveApply { apply_id: apply.id })],
    );
    assert_rejected(apply_edit_batch(&locked_source, &remove), "EDIT_REJECTED");
    assert_eq!(locked_source, before);
}

#[test]
fn item_set_prevents_implicit_deletion_and_supports_apply_preconditions() {
    let mut project = sample_project();
    let apply = apply(
        "apl_items",
        "aps_items",
        ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_video").unwrap()],
        },
    );
    project.project.sequences[0].applies.push(apply.clone());
    let remove_clip = batch(
        "op_apply_remove_item",
        &project,
        vec![EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_video").unwrap(),
        }],
    );
    assert_rejected(apply_edit_batch(&project, &remove_clip), "EDIT_REJECTED");

    let mut replacement = apply.clone();
    replacement.enabled = false;
    let mut guarded = batch(
        "op_apply_guarded",
        &project,
        vec![structure(StructureEdit::SetApply {
            apply_id: apply.id.clone(),
            apply: Box::new(replacement),
        })],
    );
    guarded.preconditions = vec![
        Precondition::ApplyExists {
            apply_id: apply.id.clone(),
        },
        Precondition::ApplyEquals {
            apply_id: apply.id.clone(),
            apply: Box::new(apply.clone()),
        },
    ];
    assert!(matches!(
        apply_edit_batch(&project, &guarded),
        EditOutcome::Applied { .. }
    ));

    let mut wrong = apply;
    wrong.enabled = false;
    guarded.preconditions[1] = Precondition::ApplyEquals {
        apply_id: wrong.id.clone(),
        apply: Box::new(wrong),
    };
    assert!(matches!(
        apply_edit_batch(&project, &guarded),
        EditOutcome::Conflict { .. }
    ));
}

fn structure(edit: StructureEdit) -> EditOperation {
    EditOperation::EditStructure { edit }
}
