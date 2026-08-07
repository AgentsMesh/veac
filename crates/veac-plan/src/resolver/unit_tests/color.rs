use super::support::*;
use crate::{canonical::*, resolve, ResolvedColorStage, ResolvedInputKind, ResolvedLutKind};

#[test]
fn every_color_stage_and_lut_kind_becomes_owned_plan_data() {
    let mut value = project();
    value.project.materials.extend([
        lut("med_zlut1", MaterialKind::Lut1d),
        lut("med_zlut3", MaterialKind::Lut3d),
    ]);
    value.project.sequences[0].tracks[0].clips[0].visual = Some(graded_visual());

    let plan = resolve(&value, None).unwrap().remove(0);
    let pipeline = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .color_pipeline
        .as_ref()
        .unwrap();
    assert_eq!(pipeline.stages.len(), 6);
    assert!(matches!(
        pipeline.stages[0],
        ResolvedColorStage::Basic { .. }
    ));
    assert!(matches!(pipeline.stages[1], ResolvedColorStage::Hsl { .. }));
    assert!(matches!(
        pipeline.stages[2],
        ResolvedColorStage::Curves { .. }
    ));
    assert!(matches!(
        pipeline.stages[3],
        ResolvedColorStage::Wheels { .. }
    ));
    let kinds: Vec<_> = pipeline.stages[4..]
        .iter()
        .map(|stage| match stage {
            ResolvedColorStage::Lut { application } => application.kind,
            _ => panic!("expected LUT stage"),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            ResolvedLutKind::OneDimensional,
            ResolvedLutKind::ThreeDimensional
        ]
    );
    assert_eq!(
        plan.inputs
            .iter()
            .filter(|input| matches!(input.kind, ResolvedInputKind::Resource { .. }))
            .count(),
        2
    );
}

fn graded_visual() -> VisualProperties {
    let mut visual = visual_properties();
    let wheel = ColorWheel {
        red: 0.1,
        green: 0.0,
        blue: -0.1,
    };
    visual.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![
            ColorStage::Basic {
                adjustment: BasicColorAdjustment {
                    exposure_stops: 0.5,
                    temperature_kelvin: 6_500.0,
                    tint: 0.1,
                    highlights: -0.1,
                    shadows: 0.1,
                    fade: 0.05,
                },
            },
            ColorStage::Hsl {
                adjustment: HslAdjustment {
                    range: HueRange::Blue,
                    hue_degrees: 5.0,
                    saturation: 0.2,
                    lightness: -0.1,
                },
            },
            ColorStage::Curves {
                curves: ColorCurves {
                    luma: Some(ToneCurve {
                        points: vec![
                            CurvePoint {
                                input: 0.0,
                                output: 0.0,
                            },
                            CurvePoint {
                                input: 1.0,
                                output: 1.0,
                            },
                        ],
                        interpolation: ToneCurveInterpolation::Monotonic,
                    }),
                    red: None,
                    green: None,
                    blue: None,
                },
            },
            ColorStage::Wheels {
                wheels: LiftGammaGain {
                    lift: wheel,
                    gamma: wheel,
                    gain: wheel,
                },
            },
            lut_stage("med_zlut1", LutInterpolation::Linear),
            lut_stage("med_zlut3", LutInterpolation::Tetrahedral),
        ],
    });
    visual
}

fn lut_stage(id: &str, interpolation: LutInterpolation) -> ColorStage {
    ColorStage::Lut {
        application: LutApplication {
            material_id: MaterialId::new(id).unwrap(),
            interpolation,
        },
    }
}

fn lut(id: &str, kind: MaterialKind) -> Material {
    Material {
        id: MaterialId::new(id).unwrap(),
        kind,
        source: MaterialSource::File {
            uri: format!("looks/{id}.cube"),
        },
        identity: Some(identity(if kind == MaterialKind::Lut1d {
            '1'
        } else {
            '3'
        })),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        authorship: None,
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
