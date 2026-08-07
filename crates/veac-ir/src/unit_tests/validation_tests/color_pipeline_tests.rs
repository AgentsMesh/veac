use super::*;

#[test]
fn complete_ordered_color_pipeline_with_lut_validates() {
    let mut project = sample_project();
    project.project.materials.push(lut_material(
        "med_look",
        MaterialKind::Lut3d,
        "looks/film.cube",
    ));
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline = Some(ColorPipeline {
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
                curves: luma_curve(),
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
                    interpolation: LutInterpolation::Tetrahedral,
                },
            },
        ],
    });
    validate(&project).unwrap();
}

#[test]
fn color_stages_reject_non_finite_ranges_unsorted_curves_and_bad_luts() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![
            ColorStage::Basic {
                adjustment: BasicColorAdjustment {
                    exposure_stops: f64::NAN,
                    temperature_kelvin: 500.0,
                    tint: 2.0,
                    highlights: 0.0,
                    shadows: 0.0,
                    fade: -1.0,
                },
            },
            ColorStage::Hsl {
                adjustment: HslAdjustment {
                    range: HueRange::Blue,
                    hue_degrees: 200.0,
                    saturation: 0.0,
                    lightness: 0.0,
                },
            },
            ColorStage::Curves {
                curves: ColorCurves {
                    luma: Some(ToneCurve {
                        points: vec![point(0.8, 0.2), point(0.2, 0.8)],
                        interpolation: ToneCurveInterpolation::Natural,
                    }),
                    red: None,
                    green: None,
                    blue: None,
                },
            },
            ColorStage::Wheels {
                wheels: LiftGammaGain {
                    lift: wheel(2.0, 0.0, 0.0),
                    gamma: wheel(0.0, 0.0, 0.0),
                    gain: wheel(0.0, 0.0, 0.0),
                },
            },
            ColorStage::Lut {
                application: LutApplication {
                    material_id: MaterialId::new("med_video").unwrap(),
                    interpolation: LutInterpolation::Linear,
                },
            },
        ],
    });
    let codes = validation_codes(&project);
    for code in [
        "COLOR_BASIC",
        "COLOR_HSL",
        "COLOR_CURVES",
        "COLOR_WHEELS",
        "LUT_MATERIAL_KIND",
    ] {
        assert_code(&codes, code);
    }

    if let ColorStage::Lut { application } = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap()
        .stages
        .last_mut()
        .unwrap()
    {
        application.material_id = MaterialId::new("med_missing_lut").unwrap();
    }
    assert_code(&validation_codes(&project), "LUT_MATERIAL_NOT_FOUND");
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}

fn luma_curve() -> ColorCurves {
    ColorCurves {
        luma: Some(ToneCurve {
            points: vec![point(0.0, 0.0), point(0.5, 0.55), point(1.0, 1.0)],
            interpolation: ToneCurveInterpolation::Monotonic,
        }),
        red: None,
        green: None,
        blue: None,
    }
}

fn point(input: f64, output: f64) -> CurvePoint {
    CurvePoint { input, output }
}

fn wheel(red: f64, green: f64, blue: f64) -> ColorWheel {
    ColorWheel { red, green, blue }
}

fn lut_material(id: &str, kind: MaterialKind, uri: &str) -> Material {
    Material {
        id: MaterialId::new(id).unwrap(),
        kind,
        source: MaterialSource::File {
            uri: uri.to_owned(),
        },
        identity: Some(MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: "b".repeat(64),
        }),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        authorship: None,
    }
}
