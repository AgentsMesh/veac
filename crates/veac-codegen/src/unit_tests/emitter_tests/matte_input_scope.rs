use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::{ResolvedInputKind, ResolvedMatte};

use super::support::{output_bindings, resolved, test_font_path, text_fixture};

#[test]
fn hidden_caption_matte_transitively_requires_its_font() {
    let mut plan = resolved(&text_fixture(true));
    plan.output.video_deliverable_mut().unwrap().captions = CaptionOutput::Discard;
    let caption = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips[0]
        .id
        .clone();
    let target_range = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_video")
        .unwrap()
        .clips[0]
        .record_range;
    plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips[0]
        .record_range = target_range;
    plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_video")
        .unwrap()
        .clips[0]
        .visual
        .as_mut()
        .unwrap()
        .track_matte = Some(ResolvedMatte {
        relation_id: RelationId::new("rel_caption_matte").unwrap(),
        source_clip_id: caption,
        mode: TrackMatteMode::Alpha,
        invert: false,
    });
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        if matches!(input.kind, ResolvedInputKind::Media { .. }) {
            bindings
                .bind_original(input, format!("/tmp/{}.mov", input.id).into())
                .unwrap();
        }
    }
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "INPUT_BINDING_MISSING"));

    let font = plan
        .inputs
        .iter()
        .find(|input| matches!(input.kind, ResolvedInputKind::Font { .. }))
        .unwrap();
    bindings.bind_original(font, test_font_path()).unwrap();
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 2);
}
