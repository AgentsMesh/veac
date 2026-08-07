use crate::test_support::time;

use super::*;

#[test]
fn locked_tracks_reject_all_extended_timeline_and_stack_operations() {
    let mut project = magnetic_project();
    project.project.sequences[0].tracks[0].state.locked = true;
    let mut inserted = project.project.sequences[0].tracks[0].clips[0].clone();
    inserted.id = ItemId::new("itm_locked_ripple").unwrap();
    inserted.record_range = range(600, 60);
    let blur = EffectInstance {
        id: EffectId::new("fx_locked_blur").unwrap(),
        enabled: true,
        enable_range: None,
        effect: Effect::VideoBlur {
            radius: Animatable::constant(2.0),
        },
    };
    let id = ItemId::new("itm_video").unwrap();
    let operations = vec![
        EditOperation::RippleInsert {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_video").unwrap(),
            clip: Box::new(inserted),
        },
        EditOperation::RippleDelete {
            clip_id: id.clone(),
        },
        EditOperation::TrimClip {
            clip_id: id.clone(),
            edge: TrimEdge::Out,
            delta: time(-1),
            ripple: true,
        },
        EditOperation::SplitClip {
            clip_id: id.clone(),
            at: time(300),
            right_clip_id: ItemId::new("itm_locked_split").unwrap(),
            relation_fragments: vec![],
        },
        EditOperation::SlipClip {
            clip_id: id.clone(),
            source_delta: time(1),
        },
        EditOperation::RollEdit {
            left_clip_id: id.clone(),
            right_clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(1),
        },
        EditOperation::SlideClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(1),
        },
        EditOperation::SetVisual {
            clip_id: id.clone(),
            visual: None,
        },
        EditOperation::SetAudio {
            clip_id: id.clone(),
            audio: None,
        },
        EditOperation::SetTransition {
            clip_id: id.clone(),
            transition: None,
        },
        EditOperation::AddEffect {
            clip_id: id.clone(),
            effect: blur,
            before_id: None,
            after_id: None,
        },
        EditOperation::RemoveEffect {
            clip_id: id.clone(),
            effect_id: EffectId::new("fx_missing").unwrap(),
        },
        EditOperation::MoveEffect {
            clip_id: id,
            effect_id: EffectId::new("fx_missing").unwrap(),
            before_id: None,
            after_id: None,
        },
    ];
    for (index, operation) in operations.into_iter().enumerate() {
        let edit = batch(
            &format!("op_locked_extended_{index}"),
            &project,
            vec![operation],
        );
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
}
