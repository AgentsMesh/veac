use tempfile::tempdir;

use super::support::*;

const SEGMENT_MS: i64 = 500;

#[test]
fn every_extended_blend_mode_produces_expected_real_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("blend-modes.mp4");
    let cases = cases();
    let duration = SEGMENT_MS * cases.len() as i64;
    let mut canonical = project(false);
    let background = solid_clip("itm_blend_bg", color(48, 128, 208), 0, duration);
    let layers = cases
        .iter()
        .enumerate()
        .map(|(index, case)| blend_clip(index, case.mode))
        .collect();
    canonical.project.sequences[0].tracks.extend([
        track("trk_blend_bg", TrackKind::Video, 0, vec![background]),
        track("trk_blend_layers", TrackKind::Visual, 1, layers),
    ]);

    render(canonical, &BTreeMap::new(), &output);
    assert_media_contract(&output, 0, duration as f64 / 1_000.0);
    for (index, case) in cases.iter().enumerate() {
        let second = (index as f64 + 0.5) * SEGMENT_MS as f64 / 1_000.0;
        let actual = rgb_at(&output, second, WIDTH / 2, HEIGHT / 2);
        assert_rgb_near(case.name, actual, case.expected, 8);
    }
}

struct BlendCase {
    name: &'static str,
    mode: BlendMode,
    expected: [u8; 3],
}

fn cases() -> [BlendCase; 9] {
    [
        case("overlay", BlendMode::Overlay, [48, 160, 184]),
        case("darken", BlendMode::Darken, [48, 128, 64]),
        case("lighten", BlendMode::Lighten, [128, 160, 208]),
        case("color dodge", BlendMode::ColorDodge, [158, 64, 255]),
        case("color burn", BlendMode::ColorBurn, [170, 64, 79]),
        case("hard light", BlendMode::HardLight, [48, 160, 104]),
        case("soft light", BlendMode::SoftLight, [48, 144, 189]),
        case("difference", BlendMode::Difference, [80, 32, 144]),
        case("exclusion", BlendMode::Exclusion, [128, 128, 168]),
    ]
}

fn case(name: &'static str, mode: BlendMode, expected: [u8; 3]) -> BlendCase {
    BlendCase {
        name,
        mode,
        expected,
    }
}

fn blend_clip(index: usize, mode: BlendMode) -> Clip {
    let mut clip = solid_clip(
        &format!("itm_blend_{index}"),
        color(128, 160, 64),
        index as i64 * SEGMENT_MS,
        SEGMENT_MS,
    );
    let mut visual = full_visual();
    visual.compositing.blend_mode = mode;
    clip.visual = Some(visual);
    clip
}

fn assert_rgb_near(name: &str, actual: [u8; 3], expected: [u8; 3], tolerance: u8) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(left, right)| left.abs_diff(right) <= tolerance),
        "{name}: actual={actual:?}, expected={expected:?}, tolerance={tolerance}"
    );
}
