use veac_plan::canonical::*;
use veac_plan::{ResolvedApply, ResolvedApplyTarget};

use super::super::support::assert_code;
use super::super::{compile_binding, evaluate_binding};
use super::clock_plan;
use crate::emitter::process_owner::ProcessOwner;

#[test]
fn compile_and_evaluate_cover_detached_and_apply_owner_contracts() {
    let (plan, binding) = clock_plan(TemporalClock::SequenceTime);
    let mut detached = plan.sequences[0].tracks[0].clips[0].clone();
    detached.id = ItemId::new("itm_temporal_detached").unwrap();
    let owner = ProcessOwner::clip(&detached);
    assert_code(
        compile_binding(&plan, &binding, owner, "t"),
        "TEMPORAL_BACKEND_CONTRACT",
    );
    assert_eq!(
        evaluate_binding(&plan, &binding, owner, 0.0)
            .unwrap_err()
            .code,
        "TEMPORAL_BACKEND_CONTRACT"
    );

    for clock in [TemporalClock::SequenceTime, TemporalClock::Frame] {
        let (mut plan, binding) = clock_plan(clock);
        push_apply(&mut plan);
        let owner = ProcessOwner::apply(&plan.sequences[0].applies[0]);
        compile_binding(&plan, &binding, owner, "t").unwrap();
        evaluate_binding(&plan, &binding, owner, 0.1).unwrap();
    }
    for clock in [TemporalClock::Progress, TemporalClock::SourceTime] {
        let (mut plan, binding) = clock_plan(clock);
        push_apply(&mut plan);
        let owner = ProcessOwner::apply(&plan.sequences[0].applies[0]);
        assert_code(
            compile_binding(&plan, &binding, owner, "t"),
            "TEMPORAL_BACKEND_CONTRACT",
        );
        assert_eq!(
            evaluate_binding(&plan, &binding, owner, 0.1)
                .unwrap_err()
                .code,
            "TEMPORAL_BACKEND_CONTRACT"
        );
    }
}

fn push_apply(plan: &mut veac_plan::ResolvedRenderPlan) {
    let range = plan.sequences[0].tracks[0].clips[0].record_range;
    plan.sequences[0].applies.push(ResolvedApply {
        id: ApplyId::new("apl_temporal_owner").unwrap(),
        source_order: 0,
        record_range: range,
        target: ResolvedApplyTarget::ItemSet { items: Vec::new() },
        stages: Vec::new(),
        mix: ApplyMix::default(),
        matte: None,
    });
}
