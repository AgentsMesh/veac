use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_plan::canonical::*;
use veac_plan::ResolvedMatte;

use super::composition_advanced::advanced_plan;
use super::support::{
    add_transition, bindings, emit_video_command, resolved, text_fixture, time, visual,
};

#[test]
fn text_can_be_a_matte_source_or_target() {
    for (caption, text_is_source) in [(false, true), (false, false), (true, true), (true, false)] {
        let mut project = text_fixture(caption);
        let video_id = project.project.sequences[0].tracks[0].clips[0].id.clone();
        project.project.sequences[0].tracks[0].clips[0].visual = Some(plain_visual());
        let text = &mut project.project.sequences[0].tracks[1].clips[0];
        text.record_range.start = time(0);
        let text_id = text.id.clone();
        let (producer_id, consumer_id) = if text_is_source {
            (text_id, video_id)
        } else {
            (video_id, text_id)
        };
        project.project.relations.push(Relation {
            id: RelationId::new("rel_text_matte").unwrap(),
            sequence_id: SequenceId::new("seq_main").unwrap(),
            kind: RelationKind::Matte {
                producer: RelationEndpoint::item(producer_id),
                consumer: RelationEndpoint::item(consumer_id),
                parameters: MatteRelationParameters {
                    mode: TrackMatteMode::Alpha,
                    invert: false,
                },
            },
        });
        let plan = resolved(&project);
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        for marker in ["textassv", "mattetrimv", "mattemergev"] {
            assert!(graph.contains(marker), "missing {marker}: {graph}");
        }
    }
}

#[test]
fn text_transition_endpoints_use_the_complete_layer_renderer() {
    for caption in [false, true] {
        let mut project = text_fixture(caption);
        let track = &mut project.project.sequences[0].tracks[1];
        track.clips[0].record_range.start = time(0);
        let visual = track.clips[0].visual.as_mut().unwrap();
        visual.compositing.z_index = 0;
        visual.compositing.blend_mode = BlendMode::Normal;
        let transition = Transition {
            kind: TransitionKind::Dissolve,
            duration: time(120),
            alignment: TransitionAlignment::Centered,
        };
        let outgoing_id = track.clips[0].id.clone();
        let mut incoming = track.clips[0].clone();
        incoming.id = ItemId::new("itm_text_transition_in").unwrap();
        incoming.record_range.start = time(600);
        incoming.visual = Some(plain_visual());
        incoming.effects.clear();
        track.clips.push(incoming);
        add_transition(
            &mut project,
            "seq_main",
            outgoing_id.as_str(),
            "itm_text_transition_in",
            transition,
        );
        let plan = resolved(&project);
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        assert!(graph.matches("textassv").count() >= 4, "graph={graph}");
        assert!(graph.contains("transitionv"), "graph={graph}");
    }
}

fn plain_visual() -> VisualProperties {
    let mut value = visual();
    value.frame = None;
    value.transform.position = Animatable::constant(Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    });
    value.opacity = Animatable::constant(1.0);
    value.compositing = Compositing {
        z_index: 0,
        blend_mode: BlendMode::Normal,
    };
    value.masks.clear();
    value.card = None;
    value
}

#[test]
fn excessive_matte_depth_is_a_typed_error_before_recursion() {
    let mut plan = advanced_plan();
    let track = &mut plan.sequences[0].tracks[1];
    for index in 1..=65 {
        let mut clip = track.clips[0].clone();
        clip.id = ItemId::new(format!("itm_matte_depth_{index:03}")).unwrap();
        clip.source_order = index;
        clip.visual.as_mut().unwrap().track_matte = None;
        track.clips.push(clip);
    }
    for index in 0..track.clips.len() - 1 {
        let source_clip_id = track.clips[index + 1].id.clone();
        track.clips[index].visual.as_mut().unwrap().track_matte = Some(ResolvedMatte {
            relation_id: RelationId::new(format!("rel_matte_depth_{index:03}")).unwrap(),
            source_clip_id,
            mode: TrackMatteMode::Alpha,
            invert: false,
        });
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        emit_video_command(&plan, &bindings(&plan))
    }));
    let error = result.expect("preflight must not recurse").unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == "PLAN_MATTE_DEPTH_EXCEEDED"),
        "error={error}"
    );
}
