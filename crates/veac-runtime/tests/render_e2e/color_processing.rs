use tempfile::tempdir;

use super::support::*;

#[test]
fn advanced_color_stages_change_real_output_pixels() {
    let temp = tempdir().unwrap();
    let source_color = color(48, 80, 120);
    let baseline = temp.path().join("baseline.mp4");
    render_color(source_color, None, None, None, &baseline);
    let graded = temp.path().join("graded.mp4");
    render_color(
        source_color,
        Some(ColorPipeline {
            input: rec709(),
            working: rec709(),
            output: rec709(),
            stages: vec![
                ColorStage::Basic {
                    adjustment: BasicColorAdjustment {
                        exposure_stops: 0.75,
                        temperature_kelvin: 6_000.0,
                        tint: 0.1,
                        highlights: 0.2,
                        shadows: 0.1,
                        fade: 0.05,
                    },
                },
                ColorStage::Hsl {
                    adjustment: HslAdjustment {
                        range: HueRange::Blue,
                        hue_degrees: 10.0,
                        saturation: 0.3,
                        lightness: 0.1,
                    },
                },
                ColorStage::Curves {
                    curves: ColorCurves {
                        luma: Some(curve()),
                        red: None,
                        green: None,
                        blue: None,
                    },
                },
                ColorStage::Wheels {
                    wheels: LiftGammaGain {
                        lift: wheel(0.05, 0.0, -0.05),
                        gamma: wheel(0.0, 0.03, 0.0),
                        gain: wheel(0.05, 0.05, 0.0),
                    },
                },
            ],
        }),
        None,
        None,
        &graded,
    );
    let baseline_pixel = rgb_at(&baseline, 0.5, WIDTH / 2, HEIGHT / 2);
    let graded_pixel = rgb_at(&graded, 0.5, WIDTH / 2, HEIGHT / 2);
    assert_ne!(graded_pixel, baseline_pixel);
    assert!(
        graded_pixel
            .iter()
            .map(|value| u16::from(*value))
            .sum::<u16>()
            > baseline_pixel
                .iter()
                .map(|value| u16::from(*value))
                .sum::<u16>(),
        "baseline={baseline_pixel:?}, graded={graded_pixel:?}"
    );
}

#[test]
fn colorspace_conversion_survives_encode_with_explicit_output_metadata() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("metadata.mp4");
    render_color(
        color(80, 100, 120),
        Some(ColorPipeline {
            input: rec709(),
            working: ColorSpace {
                transfer: ColorTransfer::Linear,
                range: ColorRange::Full,
                ..rec709()
            },
            output: rec709(),
            stages: vec![],
        }),
        None,
        Some(rec709()),
        &output,
    );
    let metadata = color_metadata(&output);
    let stream = &metadata["streams"][0];
    assert_eq!(stream["color_space"], "bt709");
    assert_eq!(stream["color_transfer"], "bt709");
    assert_eq!(stream["color_primaries"], "bt709");
    assert_eq!(stream["color_range"], "tv");
}

#[test]
fn pq_and_hlg_working_spaces_execute_through_real_ffmpeg() {
    let temp = tempdir().unwrap();
    for (transfer, name) in [
        (ColorTransfer::Smpte2084, "pq"),
        (ColorTransfer::AribStdB67, "hlg"),
    ] {
        let output = temp.path().join(format!("{name}-working.mp4"));
        let rendered = render_color(
            color(80, 100, 120),
            Some(ColorPipeline {
                input: rec709(),
                working: ColorSpace {
                    primaries: ColorPrimaries::Bt2020,
                    transfer,
                    matrix: ColorMatrix::Bt2020Ncl,
                    range: ColorRange::Limited,
                },
                output: rec709(),
                stages: vec![],
            }),
            None,
            None,
            &output,
        );
        assert!(rendered
            .command
            .filter_graph
            .as_deref()
            .is_some_and(|graph| graph.contains("zscale=")));
        assert!(output.metadata().unwrap().len() > 0);
    }
}

fn curve() -> ToneCurve {
    ToneCurve {
        points: vec![
            CurvePoint {
                input: 0.0,
                output: 0.0,
            },
            CurvePoint {
                input: 0.5,
                output: 0.6,
            },
            CurvePoint {
                input: 1.0,
                output: 1.0,
            },
        ],
        interpolation: ToneCurveInterpolation::Monotonic,
    }
}

fn wheel(red: f64, green: f64, blue: f64) -> ColorWheel {
    ColorWheel { red, green, blue }
}
