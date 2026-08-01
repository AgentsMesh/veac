mod support;

use std::collections::{BTreeMap, BTreeSet};

use support::*;
use veac_plan::canonical::*;
use veac_plan::{required_material_ids_one, resolve_one, ResolvedApplyOperation};

#[test]
fn apply_stage_lut_is_required_only_when_its_absolute_window_hits_the_target() {
    let mut envelope = project();
    envelope
        .project
        .materials
        .extend([lut("med_lut_active", true), lut("med_lut_inactive", false)]);
    let target = &mut envelope.project.sequences[0].tracks[0].clips[0];
    target.record_range.duration = time(300);
    target.visual = Some(visual_properties());
    let mut extension = generated_clip("itm_extension", Generator::Transparent, 500);
    extension.record_range.duration = time(100);
    extension.visual = Some(visual_properties());
    envelope.project.sequences[0].tracks.push(track(
        "trk_extension",
        TrackKind::Visual,
        1,
        vec![extension],
    ));
    envelope.project.sequences[0].applies.push(Apply {
        id: ApplyId::new("apl_grade").unwrap(),
        enabled: true,
        record_range: range(0, 600),
        target: ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
        stages: vec![
            stage("aps_active", range(0, 100), "med_lut_active"),
            stage("aps_inactive", range(400, 100), "med_lut_inactive"),
        ],
        mix: ApplyMix::default(),
    });

    let config_id = envelope.project.render_configs[0].id.clone();
    let required = required_material_ids_one(&envelope, &config_id).unwrap();
    assert_eq!(
        required,
        BTreeSet::from([
            MaterialId::new("med_lut_active").unwrap(),
            MaterialId::new("med_video").unwrap(),
        ]),
    );
    let plan = resolve_one(&envelope, &config_id).unwrap();
    let planned: BTreeSet<_> = plan
        .inputs
        .iter()
        .filter_map(|input| input.material_id.clone())
        .collect();
    assert_eq!(planned, required);
    let apply = &plan
        .sequences
        .iter()
        .find(|sequence| sequence.id.as_str() == "seq_main")
        .unwrap()
        .applies[0];
    assert_eq!(apply.stages.len(), 1);
    assert!(matches!(
        apply.stages[0].operation,
        ResolvedApplyOperation::Color { .. }
    ));
}

fn stage(id: &str, active_range: TimeRange, material: &str) -> ApplyStage {
    ApplyStage {
        id: ApplyStageId::new(id).unwrap(),
        enabled: true,
        active_range: Some(active_range),
        operation: ApplyOperation::Color {
            pipeline: ColorPipeline {
                input: rec709(),
                working: rec709(),
                output: rec709(),
                stages: vec![ColorStage::Lut {
                    application: LutApplication {
                        material_id: MaterialId::new(material).unwrap(),
                        interpolation: LutInterpolation::Tetrahedral,
                    },
                }],
            },
        },
    }
}

fn lut(id: &str, hydrated: bool) -> Material {
    Material {
        id: MaterialId::new(id).unwrap(),
        kind: MaterialKind::Lut3d,
        source: MaterialSource::File {
            uri: format!("looks/{id}.cube"),
        },
        identity: hydrated.then(|| identity('c')),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        metadata: BTreeMap::new(),
    }
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
