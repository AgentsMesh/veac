use tempfile::tempdir;

use super::support::*;

#[test]
fn affine_rgb_matrix_cross_channel_and_offset_change_real_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("color-matrix.mp4");
    let mut value = project(false);
    let mut clip = solid_clip("itm_matrix", color(255, 0, 0), 0, 1_000);
    let mut visual = full_visual();
    visual.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![ColorStage::Matrix {
            adjustment: RgbMatrixAdjustment {
                matrix: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                offset: [0.0, 0.0, 0.25],
            },
        }],
    });
    clip.visual = Some(visual);
    value.project.sequences[0]
        .tracks
        .push(track("trk_matrix", TrackKind::Visual, 0, vec![clip]));

    let rendered = render(value, &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("format=gbrapf32le"), "graph={graph}");
    assert!(graph.contains("+0.25\\,0\\,1)"), "graph={graph}");
    let pixel = rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2);
    assert!(
        pixel[0] < 35 && pixel[1] > 170 && pixel[2] > 30 && pixel[2] < 100,
        "matrix pixel={pixel:?}"
    );
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
