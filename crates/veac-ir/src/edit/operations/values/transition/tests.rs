use super::*;
use crate::test_support::{sample_project, time};

fn transition(kind: TransitionKind) -> Transition {
    Transition {
        kind,
        duration: time(120),
        alignment: TransitionAlignment::Centered,
    }
}

#[test]
fn updates_an_existing_transition_and_rejects_a_terminal_source() {
    let mut envelope = sample_project();
    let track = &mut envelope.project.sequences[0].tracks[0];
    let first = track.clips[0].id.clone();
    let last = ItemId::new("itm_clip_002").expect("last clip id");
    let mut second = track.clips[0].clone();
    second.id = last.clone();
    second.record_range.start = track.clips[0].record_range.end().expect("first clip end");
    track.clips.push(second);
    let mut changed = ChangeSet::default();

    set(&mut envelope.project, &first, &None, &mut changed).expect("missing transition is a no-op");
    envelope.project.sequences[0].tracks[0].state.locked = true;
    assert!(set(
        &mut envelope.project,
        &first,
        &Some(transition(TransitionKind::Dissolve)),
        &mut changed,
    )
    .is_err());
    envelope.project.sequences[0].tracks[0].state.locked = false;

    set(
        &mut envelope.project,
        &first,
        &Some(transition(TransitionKind::Dissolve)),
        &mut changed,
    )
    .expect("create transition");
    set(
        &mut envelope.project,
        &first,
        &Some(transition(TransitionKind::Fade {
            color: FadeColor::Black,
        })),
        &mut changed,
    )
    .expect("update transition");

    assert_eq!(envelope.project.relations.len(), 1);
    let RelationKind::Transition {
        transition: existing_transition,
        ..
    } = &envelope.project.relations[0].kind
    else {
        panic!("expected transition relation");
    };
    assert_eq!(
        existing_transition.kind,
        TransitionKind::Fade {
            color: FadeColor::Black
        }
    );

    let error = set(
        &mut envelope.project,
        &last,
        &Some(transition(TransitionKind::Dissolve)),
        &mut changed,
    )
    .unwrap_err();
    assert_eq!(error.code, "EDIT_REJECTED");
}
