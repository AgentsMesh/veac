use tempfile::tempdir;

use super::support::*;

const SEGMENT_MS: i64 = 500;

#[test]
fn every_extended_mask_shape_and_inversion_affect_real_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("mask-shapes.mp4");
    let cases = cases();
    let duration = SEGMENT_MS * cases.len() as i64;
    let mut canonical = project(false);
    let background = solid_clip("itm_mask_bg", color(0, 0, 255), 0, duration);
    let foregrounds = cases
        .iter()
        .enumerate()
        .map(|(index, case)| masked_clip(index, case))
        .collect();
    canonical.project.sequences[0].tracks.extend([
        track("trk_mask_bg", TrackKind::Video, 0, vec![background]),
        track("trk_mask_shapes", TrackKind::Visual, 1, foregrounds),
    ]);

    render(canonical, &BTreeMap::new(), &output);
    assert_media_contract(&output, 0, duration as f64 / 1_000.0);
    for (index, case) in cases.iter().enumerate() {
        assert_case(&output, index, case);
    }
}

struct MaskCase {
    name: &'static str,
    shape: MaskShape,
    scale: Vec2,
    inside: (u32, u32),
    outside: (u32, u32),
    edge: (u32, u32),
    invert: bool,
}

fn cases() -> Vec<MaskCase> {
    vec![
        case(
            "linear",
            MaskShape::Linear,
            (0.60, 0.60),
            (70, 27),
            (25, 27),
            (48, 27),
            false,
        ),
        case(
            "mirror",
            MaskShape::Mirror,
            (0.60, 0.60),
            (48, 27),
            (5, 27),
            (19, 27),
            false,
        ),
        case(
            "ellipse",
            MaskShape::Ellipse,
            (0.75, 0.55),
            (48, 27),
            (93, 27),
            (84, 27),
            false,
        ),
        case(
            "rounded rectangle",
            MaskShape::RoundedRectangle { radius: 0.2 },
            (0.75, 0.55),
            (48, 27),
            (0, 0),
            (48, 12),
            false,
        ),
        case(
            "polygon",
            MaskShape::Polygon {
                points: vec![
                    Vec2 { x: 0.1, y: 0.1 },
                    Vec2 { x: 0.9, y: 0.1 },
                    Vec2 { x: 0.5, y: 0.9 },
                ],
            },
            (0.75, 0.55),
            (48, 27),
            (0, 0),
            (48, 15),
            false,
        ),
        case(
            "heart",
            MaskShape::Heart,
            (0.65, 0.65),
            (48, 27),
            (48, 3),
            (48, 11),
            false,
        ),
        case(
            "star",
            MaskShape::Star,
            (0.65, 0.65),
            (48, 27),
            (92, 27),
            (63, 27),
            false,
        ),
        case(
            "inverted ellipse",
            MaskShape::Ellipse,
            (0.75, 0.55),
            (48, 27),
            (93, 27),
            (84, 27),
            true,
        ),
    ]
}

fn case(
    name: &'static str,
    shape: MaskShape,
    scale: (f64, f64),
    inside: (u32, u32),
    outside: (u32, u32),
    edge: (u32, u32),
    invert: bool,
) -> MaskCase {
    MaskCase {
        name,
        shape,
        scale: Vec2 {
            x: scale.0,
            y: scale.1,
        },
        inside,
        outside,
        edge,
        invert,
    }
}

fn masked_clip(index: usize, case: &MaskCase) -> Clip {
    let mut clip = solid_clip(
        &format!("itm_mask_{index}"),
        color(255, 0, 0),
        index as i64 * SEGMENT_MS,
        SEGMENT_MS,
    );
    let mut visual = full_visual();
    let mut mask = default_mask(case.shape.clone());
    mask.scale = Animatable::constant(case.scale);
    mask.feather_pixels = Animatable::constant(2.0);
    mask.invert = case.invert;
    visual.masks.push(mask);
    clip.visual = Some(visual);
    clip
}

fn assert_case(output: &Path, index: usize, case: &MaskCase) {
    let second = (index as f64 + 0.5) * SEGMENT_MS as f64 / 1_000.0;
    let inside = rgb_at(output, second, case.inside.0, case.inside.1);
    let outside = rgb_at(output, second, case.outside.0, case.outside.1);
    let edge = rgb_at(output, second, case.edge.0, case.edge.1);
    if case.invert {
        assert_blue(case.name, inside);
        assert_red(case.name, outside);
    } else {
        assert_red(case.name, inside);
        assert_blue(case.name, outside);
    }
    assert!(
        edge[0] > 35 && edge[2] > 35,
        "{} edge must mix foreground/background: {edge:?}",
        case.name
    );
}

fn assert_red(name: &str, pixel: [u8; 3]) {
    assert!(pixel[0] > 170 && pixel[2] < 70, "{name} red={pixel:?}");
}

fn assert_blue(name: &str, pixel: [u8; 3]) {
    assert!(
        pixel[2] > 140 && u16::from(pixel[2]) > u16::from(pixel[0]) + 70,
        "{name} blue={pixel:?}"
    );
}
