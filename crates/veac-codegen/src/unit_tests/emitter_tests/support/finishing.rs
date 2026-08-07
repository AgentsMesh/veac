use veac_plan::canonical::*;

use super::{file_identity, fixture, visual};

pub fn graded_project(kind: MaterialKind, interpolation: LutInterpolation) -> ProjectEnvelope {
    let mut project = fixture();
    project.project.materials.push(Material {
        id: MaterialId::new("med_look").unwrap(),
        kind,
        source: MaterialSource::File {
            uri: "looks/look.cube".to_owned(),
        },
        identity: Some(file_identity(&lut_fixture(kind))),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        authorship: None,
    });
    let mut properties = visual();
    properties.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: ColorSpace {
            transfer: ColorTransfer::Linear,
            range: ColorRange::Full,
            ..rec709()
        },
        output: rec709(),
        stages: vec![
            ColorStage::Basic {
                adjustment: BasicColorAdjustment {
                    exposure_stops: 0.5,
                    temperature_kelvin: 6_000.0,
                    tint: 0.1,
                    highlights: -0.2,
                    shadows: 0.2,
                    fade: 0.1,
                },
            },
            ColorStage::Hsl {
                adjustment: HslAdjustment {
                    range: HueRange::Red,
                    hue_degrees: 5.0,
                    saturation: 0.2,
                    lightness: -0.1,
                },
            },
            ColorStage::Curves {
                curves: ColorCurves {
                    luma: Some(tone_curve(ToneCurveInterpolation::Monotonic)),
                    red: None,
                    green: None,
                    blue: None,
                },
            },
            ColorStage::Wheels {
                wheels: LiftGammaGain {
                    lift: wheel(0.05, 0.0, -0.05),
                    gamma: wheel(0.0, 0.02, 0.0),
                    gain: wheel(0.05, 0.05, 0.0),
                },
            },
            ColorStage::Lut {
                application: LutApplication {
                    material_id: MaterialId::new("med_look").unwrap(),
                    interpolation,
                },
            },
        ],
    });
    project.project.sequences[0].tracks[0].clips[0].visual = Some(properties);
    project
}

pub fn lut_fixture(kind: MaterialKind) -> std::path::PathBuf {
    let name = match kind {
        MaterialKind::Lut1d => "identity-1d.cube",
        MaterialKind::Lut3d => "identity-3d.cube",
        _ => panic!("LUT fixture requires a LUT material kind"),
    };
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

pub fn tone_curve(interpolation: ToneCurveInterpolation) -> ToneCurve {
    ToneCurve {
        points: vec![
            CurvePoint {
                input: 0.0,
                output: 0.0,
            },
            CurvePoint {
                input: 0.5,
                output: 0.55,
            },
            CurvePoint {
                input: 1.0,
                output: 1.0,
            },
        ],
        interpolation,
    }
}

fn wheel(red: f64, green: f64, blue: f64) -> ColorWheel {
    ColorWheel { red, green, blue }
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
