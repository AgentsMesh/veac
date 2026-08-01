use super::*;
use crate::unit_tests::emitter_tests::support::bindings;
use crate::unit_tests::emitter_tests::transitions::transition_plan;
use veac_plan::canonical::{DeliverableKind, ItemId, TransitionAlignment, TransitionKind};

fn build_context<'a>(
    plan: &'a veac_plan::ResolvedRenderPlan,
    execution: &'a veac_artifact::ExecutionBindings,
) -> EmitContext<'a> {
    let deliverable = &plan.output.deliverables[0];
    let alpha = match &deliverable.kind {
        DeliverableKind::Video(video) => video.video.alpha,
        _ => panic!("expected video deliverable"),
    };
    EmitContext::new_visual(plan, execution, deliverable, alpha).expect("context")
}

#[test]
fn transition_backend_rejects_missing_and_nonvisual_endpoints() {
    let plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    let execution = bindings(&plan);
    let mut context = build_context(&plan, &execution);
    let mut sequence = plan.sequences[0].clone();
    sequence.tracks[0].clips[0].visual = None;
    let track = &sequence.tracks[0];
    let transition = &track.transitions[0];
    let error = transition::compose(
        &mut context,
        &sequence,
        "base".to_owned(),
        track,
        transition,
    )
    .unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TRANSITION_SOURCE_INVALID");

    let plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    let execution = bindings(&plan);
    let mut context = build_context(&plan, &execution);
    let mut sequence = plan.sequences[0].clone();
    sequence.tracks[0].transitions[0].outgoing_clip_id =
        ItemId::new("itm_missing").expect("item id");
    let track = &sequence.tracks[0];
    let transition = &track.transitions[0];
    let error = transition::compose(
        &mut context,
        &sequence,
        "base".to_owned(),
        track,
        transition,
    )
    .unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TRANSITION_PLAN_INVALID");
}
