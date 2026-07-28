use super::super::process_owner::ProcessOwner;
use super::*;
use veac_plan::canonical::{AlphaMode, LutInterpolation, MaterialKind};
use veac_plan::{PlanInputId, ResolvedInputKind, ResolvedLut, ResolvedLutKind};

#[test]
fn lut_backend_rejects_missing_resources_and_incompatible_interpolation() {
    let plan = resolved(&fixture());
    let missing = ResolvedLut {
        input_id: PlanInputId::new("pin_missing_lut").unwrap(),
        kind: ResolvedLutKind::OneDimensional,
        interpolation: LutInterpolation::Nearest,
    };
    assert!(lut_error(&plan, &missing)
        .to_string()
        .contains("COLOR_RESOURCE_INVALID"));

    let mut plan = resolved(&fixture());
    let input_id = PlanInputId::new("pin_lut_1d").unwrap();
    let mut input = plan.inputs[0].clone();
    input.id = input_id.clone();
    input.kind = ResolvedInputKind::Resource {
        material_kind: MaterialKind::Lut1d,
    };
    input.video = None;
    input.audio = None;
    plan.inputs.push(input);
    let incompatible = ResolvedLut {
        input_id,
        kind: ResolvedLutKind::OneDimensional,
        interpolation: LutInterpolation::Tetrahedral,
    };
    assert!(lut_error(&plan, &incompatible)
        .to_string()
        .contains("COLOR_PROCESSING_UNSUPPORTED"));
}

#[test]
fn text_event_layers_count_background_and_independent_shadow() {
    let event_layers = super::super::text::backend::event_layers;
    assert_eq!(event_layers(false, false), 1);
    assert_eq!(event_layers(true, false), 2);
    assert_eq!(event_layers(false, true), 2);
    assert_eq!(event_layers(true, true), 3);
}

#[test]
fn multicam_backend_rejects_unknown_audio_angle() {
    let plan = multicam_plan();
    let mut source = multicam_source(&plan);
    source.switches[0].angle_id = MulticamAngleId::new("ang_missing_audio").unwrap();
    let error = multicam_direct_error(&plan, &source);
    assert!(
        error
            .to_string()
            .contains("multicam switch angle is missing"),
        "error={error}"
    );
}

fn multicam_direct_error(
    plan: &ResolvedRenderPlan,
    source: &ResolvedMulticamSource,
) -> super::super::CodegenErrors {
    let execution = bindings(plan);
    let deliverable = &plan.output.deliverables[0];
    let mut context = super::super::EmitContext {
        plan,
        bindings: &execution,
        deliverable,
        alpha: AlphaMode::Opaque,
        input_routes: Default::default(),
        canvas: super::super::Canvas::from_output(&plan.output),
        graph: Default::default(),
        filter_bindings: Vec::new(),
    };
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let audio = match &deliverable.kind {
        veac_plan::canonical::DeliverableKind::Video(video) => {
            video.audio.as_ref().expect("audio output")
        }
        _ => unreachable!(),
    };
    super::super::multicam_source::audio(&mut context, clip, source, audio).unwrap_err()
}

fn lut_error(plan: &ResolvedRenderPlan, lut: &ResolvedLut) -> super::super::CodegenErrors {
    let execution = bindings(plan);
    let deliverable = &plan.output.deliverables[0];
    let mut context = super::super::EmitContext {
        plan,
        bindings: &execution,
        deliverable,
        alpha: AlphaMode::Opaque,
        input_routes: Default::default(),
        canvas: super::super::Canvas::from_output(&plan.output),
        graph: Default::default(),
        filter_bindings: Vec::new(),
    };
    let clip = &plan.sequences[0].tracks[0].clips[0];
    super::super::color_lut::apply(
        &mut context,
        ProcessOwner::clip(clip),
        "[video_input]".to_owned(),
        lut,
    )
    .unwrap_err()
}
