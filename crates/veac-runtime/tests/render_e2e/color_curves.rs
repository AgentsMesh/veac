use tempfile::tempdir;

use super::support::*;

#[test]
fn mixed_master_and_channel_interpolations_match_equivalent_real_ffmpeg_stages() {
    let temp = tempdir().unwrap();
    let source = color(56, 112, 176);
    let baseline = temp.path().join("mixed-curves-baseline.mp4");
    render_color(source, None, None, None, &baseline);

    let mixed = temp.path().join("mixed-curves.mp4");
    render_color(
        source,
        Some(pipeline(vec![ColorStage::Curves {
            curves: mixed_curves(),
        }])),
        None,
        None,
        &mixed,
    );

    let grouped = temp.path().join("grouped-curves.mp4");
    let authored = mixed_curves();
    render_color(
        source,
        Some(pipeline(vec![
            ColorStage::Curves {
                curves: ColorCurves {
                    luma: authored.luma,
                    red: None,
                    green: authored.green,
                    blue: None,
                },
            },
            ColorStage::Curves {
                curves: ColorCurves {
                    luma: None,
                    red: authored.red,
                    green: None,
                    blue: authored.blue,
                },
            },
        ])),
        None,
        None,
        &grouped,
    );

    let baseline_pixel = rgb_at(&baseline, 0.5, WIDTH / 2, HEIGHT / 2);
    let mixed_pixel = rgb_at(&mixed, 0.5, WIDTH / 2, HEIGHT / 2);
    let grouped_pixel = rgb_at(&grouped, 0.5, WIDTH / 2, HEIGHT / 2);
    assert_ne!(
        mixed_pixel, baseline_pixel,
        "curves must affect real pixels"
    );
    assert!(
        mixed_pixel
            .into_iter()
            .zip(grouped_pixel)
            .all(|(mixed, grouped)| mixed.abs_diff(grouped) <= 2),
        "mixed={mixed_pixel:?}, grouped={grouped_pixel:?}"
    );
}

fn pipeline(stages: Vec<ColorStage>) -> ColorPipeline {
    ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages,
    }
}

fn mixed_curves() -> ColorCurves {
    ColorCurves {
        luma: Some(curve(
            ToneCurveInterpolation::Natural,
            0.25,
            0.1,
            0.65,
            0.85,
        )),
        red: Some(curve(
            ToneCurveInterpolation::Monotonic,
            0.2,
            0.5,
            0.7,
            0.75,
        )),
        green: Some(curve(ToneCurveInterpolation::Natural, 0.4, 0.65, 0.8, 0.9)),
        blue: Some(curve(
            ToneCurveInterpolation::Monotonic,
            0.25,
            0.1,
            0.6,
            0.8,
        )),
    }
}

fn curve(
    interpolation: ToneCurveInterpolation,
    input_a: f64,
    output_a: f64,
    input_b: f64,
    output_b: f64,
) -> ToneCurve {
    ToneCurve {
        points: vec![
            CurvePoint {
                input: 0.0,
                output: 0.0,
            },
            CurvePoint {
                input: input_a,
                output: output_a,
            },
            CurvePoint {
                input: input_b,
                output: output_b,
            },
            CurvePoint {
                input: 1.0,
                output: 1.0,
            },
        ],
        interpolation,
    }
}
