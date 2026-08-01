use veac_codegen::emitter::{emit_all, BackendAction};
use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedClipSource, ResolvedInputKind, ResolvedRenderPlan};

use super::support::{output_bindings, resolved, test_font_path, text_fixture};

#[test]
fn plain_caption_sidecars_need_no_media_or_font_authority() {
    for format in [CaptionSidecarFormat::Srt, CaptionSidecarFormat::WebVtt] {
        let mut plan = caption_plan(format);
        let bindings = output_bindings(&plan);
        let bundle = emit_all(&plan, &bindings).unwrap();
        assert!(bundle.protected_resources().is_empty());
        assert!(matches!(
            bundle.tasks()[0].action,
            BackendAction::WriteFile { .. }
        ));

        plan.inputs[0].observed_identity.digest = "f".repeat(64);
        assert!(emit_all(&plan, &bindings).is_ok());
    }
}

#[test]
fn ass_sidecar_protects_only_fonts_used_by_selected_cues() {
    let mut plan = caption_plan(CaptionSidecarFormat::Ass);
    normalize_ass(&mut plan);
    let selected_font = caption_font(&plan);
    add_unselected_caption_font(&mut plan);
    let mut bindings = output_bindings(&plan);
    let font = plan
        .inputs
        .iter()
        .find(|input| input.id == selected_font)
        .unwrap();
    bindings.bind_original(font, test_font_path()).unwrap();

    let bundle = emit_all(&plan, &bindings).unwrap();

    assert_eq!(bundle.protected_resources().len(), 1);
    assert_eq!(bundle.protected_resources()[0].path, test_font_path());
    assert!(matches!(
        bundle.tasks()[0].action,
        BackendAction::WriteFile { .. }
    ));
}

#[test]
fn video_caption_policy_controls_font_authority() {
    let mut plan = resolved(&text_fixture(true));
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        if matches!(input.kind, ResolvedInputKind::Media { .. }) {
            bindings
                .bind_original(input, format!("/tmp/{}.bin", input.id).into())
                .unwrap();
        }
    }
    plan.output.raster.as_mut().unwrap().captions = CaptionOutput::Discard;
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 1);
    let BackendAction::Ffmpeg(command) = &bundle.tasks()[0].action else {
        panic!("video must use FFmpeg")
    };
    assert_eq!(command.inputs.len(), 1);

    plan.output.raster.as_mut().unwrap().captions = CaptionOutput::BurnIn;
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "INPUT_BINDING_MISSING"));
}

fn caption_plan(format: CaptionSidecarFormat) -> ResolvedRenderPlan {
    let mut plan = resolved(&text_fixture(true));
    let track = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap();
    plan.output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_caption_scope").unwrap(),
        target: DeliverableTarget::File {
            name: match format {
                CaptionSidecarFormat::Srt => "scope.srt",
                CaptionSidecarFormat::WebVtt => "scope.vtt",
                CaptionSidecarFormat::Ass => "scope.ass",
            }
            .to_owned(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format,
            track_ids: vec![track.id.clone()],
        }),
    }];
    plan.output.raster = None;
    plan
}

fn normalize_ass(plan: &mut ResolvedRenderPlan) {
    let clip = plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips
        .first_mut()
        .unwrap();
    let ResolvedClipSource::Caption { content, .. } = &mut clip.source else {
        unreachable!()
    };
    content.styled_mut().unwrap().fallback_fonts.clear();
    content.styled_mut().unwrap().line_height = 1.0;
    content.styled_mut().unwrap().path = None;
    content.styled_mut().unwrap().background = None;
    content.styled_mut().unwrap().animation = None;
    let visual = clip.visual.as_mut().unwrap();
    visual.frame = None;
    visual.transform.position = Animatable::constant(Point {
        x: pixels(0.0),
        y: pixels(0.0),
    });
    visual.transform.scale = Animatable::constant(Vec2 { x: 1.0, y: 1.0 });
    visual.transform.rotation_degrees = Animatable::constant(0.0);
    visual.transform.crop = None;
    visual.opacity = Animatable::constant(1.0);
    visual.compositing.blend_mode = BlendMode::Normal;
    visual.masks.clear();
    visual.card = None;
    clip.effects.clear();
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

fn caption_font(plan: &ResolvedRenderPlan) -> PlanInputId {
    let clip = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips
        .first()
        .unwrap();
    let ResolvedClipSource::Caption { content, .. } = &clip.source else {
        unreachable!()
    };
    content.styled().unwrap().font.input_id.clone()
}

fn add_unselected_caption_font(plan: &mut ResolvedRenderPlan) {
    let mut input = plan
        .inputs
        .iter()
        .find(|value| matches!(value.kind, ResolvedInputKind::Font { .. }))
        .unwrap()
        .clone();
    input.id = PlanInputId::new("pin_unselected_caption_font").unwrap();
    let mut track = plan.sequences[0]
        .tracks
        .iter()
        .find(|value| value.id.as_str() == "trk_caption")
        .unwrap()
        .clone();
    track.id = TrackId::new("trk_unselected_caption").unwrap();
    track.order += 1;
    track.source_order = plan.sequences[0].tracks.len() as u32;
    track.clips[0].id = ItemId::new("itm_unselected_caption").unwrap();
    let ResolvedClipSource::Caption { content, .. } = &mut track.clips[0].source else {
        unreachable!()
    };
    content.styled_mut().unwrap().font.input_id = input.id.clone();
    plan.inputs.push(input);
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    plan.sequences[0].tracks.push(track);
    plan.sequences[0].tracks.sort_by(|left, right| {
        (left.order, left.source_order, &left.id).cmp(&(right.order, right.source_order, &right.id))
    });
}
