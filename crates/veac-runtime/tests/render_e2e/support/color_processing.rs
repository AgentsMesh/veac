use std::fs;
use std::process::Command;

use serde_json::Value;

use super::*;

pub(crate) fn render_color(
    color_value: Color,
    pipeline: Option<ColorPipeline>,
    lut: Option<(MaterialKind, &Path)>,
    output_color_space: Option<ColorSpace>,
    output: &Path,
) -> Rendered {
    let mut assets = BTreeMap::new();
    if let Some((_, path)) = lut {
        assets.insert("med_test_lut".to_owned(), path.to_path_buf());
    }
    let canonical = color_project(
        color_value,
        pipeline,
        lut.map(|(kind, _)| kind),
        output_color_space,
    );
    render(canonical, &assets, output)
}

pub(crate) fn color_project(
    color_value: Color,
    pipeline: Option<ColorPipeline>,
    lut: Option<MaterialKind>,
    output_color_space: Option<ColorSpace>,
) -> ProjectEnvelope {
    let mut canonical = project(false);
    let deliverable_id = canonical.project.render_configs[0].deliverables[0]
        .id
        .clone();
    canonical.project.render_configs[0]
        .video_deliverable_mut(&deliverable_id)
        .unwrap()
        .video
        .color_space = output_color_space;
    let mut clip = solid_clip("itm_color", color_value, 0, 1_000);
    let mut visual = full_frame_visual();
    visual.color_pipeline = pipeline;
    clip.visual = Some(visual);
    canonical.project.sequences[0]
        .tracks
        .push(track("trk_color", TrackKind::Video, 0, vec![clip]));
    if let Some(kind) = lut {
        canonical.project.materials.push(material(
            "med_test_lut",
            kind,
            StreamChoice::Disabled,
            StreamChoice::Disabled,
        ));
    }
    canonical
}

pub(crate) fn write_identity_lut(path: &Path, kind: MaterialKind) {
    let content = match kind {
        MaterialKind::Lut1d => {
            "TITLE \"identity\"\nLUT_1D_SIZE 2\nDOMAIN_MIN 0 0 0\nDOMAIN_MAX 1 1 1\n0 0 0\n1 1 1\n"
        }
        MaterialKind::Lut3d => {
            "TITLE \"identity\"\nLUT_3D_SIZE 2\nDOMAIN_MIN 0 0 0\nDOMAIN_MAX 1 1 1\n0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n"
        }
        _ => panic!("identity LUT requires a LUT material kind"),
    };
    fs::write(path, content).unwrap();
}

pub(crate) fn color_metadata(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=color_space,color_transfer,color_primaries,color_range",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .expect("run color metadata ffprobe");
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub(crate) fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}

fn full_frame_visual() -> VisualProperties {
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(Point {
                x: pixels(0.0),
                y: pixels(0.0),
            }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: vec![],
        card: None,
        color_pipeline: None,
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
