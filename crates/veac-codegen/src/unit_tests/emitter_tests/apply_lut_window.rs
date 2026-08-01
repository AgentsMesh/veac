use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedApplyOperation, ResolvedColorStage, ResolvedInputKind};

use super::support::{graded_project, lut_fixture, output_bindings, resolved, time};

#[test]
fn inactive_apply_lut_window_needs_no_binding_or_protected_resource() {
    let mut project = graded_project(MaterialKind::Lut3d, LutInterpolation::Tetrahedral);
    let sequence = &mut project.project.sequences[0];
    let target = &mut sequence.tracks[0].clips[0];
    target.record_range.duration = time(300);
    let pipeline = target
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .take()
        .unwrap();
    target.visual = None;
    let mut extension = sequence.tracks[0].clone();
    extension.id = TrackId::new("trk_apply_extension").unwrap();
    extension.order = 1;
    extension.placement_mode = PlacementMode::Free;
    extension.clips[0].id = ItemId::new("itm_apply_extension").unwrap();
    extension.clips[0].record_range = TimeRange::new(time(500), time(100)).unwrap();
    extension.clips[0].visual = None;
    sequence.tracks.push(extension);
    sequence.applies.push(Apply {
        id: ApplyId::new("apl_lut_window").unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(0), time(600)).unwrap(),
        target: ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
        stages: vec![ApplyStage {
            id: ApplyStageId::new("aps_active_lut").unwrap(),
            enabled: true,
            active_range: Some(TimeRange::new(time(0), time(100)).unwrap()),
            operation: ApplyOperation::Color { pipeline },
        }],
        mix: ApplyMix::default(),
    });
    let mut plan = resolved(&project);
    let active_stage = &plan.sequences[0].applies[0].stages[0];
    let ResolvedApplyOperation::Color { pipeline } = &active_stage.operation else {
        unreachable!()
    };
    let active_input = pipeline
        .stages
        .iter()
        .find_map(|stage| match stage {
            ResolvedColorStage::Lut { application } => Some(application.input_id.clone()),
            _ => None,
        })
        .unwrap();
    let mut inactive_input = plan
        .inputs
        .iter()
        .find(|input| input.id == active_input)
        .unwrap()
        .clone();
    inactive_input.id = PlanInputId::new("pin_inactive_lut").unwrap();
    inactive_input.material_id = Some(MaterialId::new("med_inactive_lut").unwrap());
    let mut inactive_stage = active_stage.clone();
    inactive_stage.id = ApplyStageId::new("aps_inactive_lut").unwrap();
    inactive_stage.active_range = TimeRange::new(time(400), time(100)).unwrap();
    let ResolvedApplyOperation::Color { pipeline } = &mut inactive_stage.operation else {
        unreachable!()
    };
    for stage in &mut pipeline.stages {
        if let ResolvedColorStage::Lut { application } = stage {
            application.input_id = inactive_input.id.clone();
        }
    }
    plan.sequences[0].applies[0].stages.push(inactive_stage);
    let inactive_id = inactive_input.id.clone();
    plan.inputs.push(inactive_input);
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        match &input.kind {
            ResolvedInputKind::Media { .. } => bindings
                .bind_original(input, format!("/tmp/{}.mov", input.id).into())
                .unwrap(),
            ResolvedInputKind::Resource { .. } if input.id != inactive_id => bindings
                .bind_original(input, lut_fixture(MaterialKind::Lut3d))
                .unwrap(),
            ResolvedInputKind::Resource { .. } => {}
            ResolvedInputKind::Font { .. } => unreachable!(),
        }
    }

    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 2);
    assert!(bindings.input(&inactive_id).is_none());
}
