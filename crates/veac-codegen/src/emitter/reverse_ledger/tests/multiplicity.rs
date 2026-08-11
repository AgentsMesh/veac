use veac_plan::canonical::*;
use veac_plan::{
    ResolvedApply, ResolvedApplyOperation, ResolvedApplyStage, ResolvedApplyTarget,
    ResolvedClipSource, ResolvedMatte,
};

use super::support::*;
use crate::unit_tests::emitter_tests::support::{add_transition, transition_visual};

#[test]
fn transition_endpoint_rebuild_is_a_distinct_reverse_instance() {
    let mut project = fixture();
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].visual = Some(transition_visual());
    let mut incoming = track.clips[0].clone();
    incoming.id = ItemId::new("itm_reverse_transition_in").unwrap();
    incoming.record_range.start = time(480);
    track.clips.push(incoming);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_reverse_transition_in",
        Transition {
            kind: TransitionKind::Dissolve,
            duration: time(120),
            alignment: TransitionAlignment::Centered,
        },
    );
    let mut plan = resolved(&project);
    reverse_clip(&mut plan.sequences[0].tracks[0].clips[0]);

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

#[test]
fn apply_band_rebuild_is_a_distinct_reverse_instance() {
    let mut plan = resolved(&fixture());
    reverse(&mut plan);
    let sequence = &mut plan.sequences[0];
    let range = sequence.tracks[0].clips[0].record_range;
    let track_id = sequence.tracks[0].id.clone();
    let item_id = sequence.tracks[0].clips[0].id.clone();
    sequence.applies.push(ResolvedApply {
        id: ApplyId::new("apl_reverse_band").unwrap(),
        source_order: 0,
        record_range: range,
        target: ResolvedApplyTarget::Layer {
            track_id,
            item_ids: vec![item_id],
            active_ranges: vec![range],
        },
        stages: vec![ResolvedApplyStage {
            id: ApplyStageId::new("aps_reverse_band").unwrap(),
            active_range: range,
            operation: ResolvedApplyOperation::Effect {
                effect: Effect::VideoBlur {
                    radius: Animatable::constant(1.0),
                },
            },
        }],
        mix: ApplyMix::default(),
        matte: None,
    });

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

#[test]
fn each_matte_consumer_rebuilds_its_reverse_source() {
    let mut plan = resolved(&fixture());
    reverse(&mut plan);
    let sequence = &mut plan.sequences[0];
    let source_id = sequence.tracks[0].clips[0].id.clone();
    let mut first = sequence.tracks[0].clone();
    first.id = TrackId::new("trk_matte_consumer_a").unwrap();
    first.order = 1;
    first.source_order = 1;
    first.clips[0].id = ItemId::new("itm_matte_consumer_a").unwrap();
    first.clips[0].source = ResolvedClipSource::Generated {
        generator: Generator::Transparent,
    };
    first.clips[0].source_mapping = None;
    first.clips[0].visual.as_mut().unwrap().track_matte = Some(matte("rel_matte_a", &source_id));
    let mut second = first.clone();
    second.id = TrackId::new("trk_matte_consumer_b").unwrap();
    second.order = 2;
    second.source_order = 2;
    second.clips[0].id = ItemId::new("itm_matte_consumer_b").unwrap();
    second.clips[0].visual.as_mut().unwrap().track_matte = Some(matte("rel_matte_b", &source_id));
    sequence.tracks.extend([first, second]);

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

fn matte(id: &str, source: &ItemId) -> ResolvedMatte {
    ResolvedMatte {
        relation_id: RelationId::new(id).unwrap(),
        source_clip_id: source.clone(),
        mode: TrackMatteMode::Alpha,
        invert: false,
    }
}
