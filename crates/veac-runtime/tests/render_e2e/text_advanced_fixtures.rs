use std::path::Path;

use veac_ir::StreamChoice;

use super::super::support::*;

pub(crate) fn advanced_style(font: &str, size: f64) -> TextStyle {
    TextStyle {
        font: font_ref(font),
        size_pixels: size,
        color: color(255, 255, 255),
        ..TextStyle::default()
    }
}

pub(super) fn font_ref(id: &str) -> FontRef {
    FontRef::Material {
        material_id: MaterialId::new(format!("med_{id}")).unwrap(),
    }
}

pub(crate) fn add_font_materials(project: &mut ProjectEnvelope, ids: &[&str]) {
    project.project.materials.extend(ids.iter().map(|id| {
        material(
            &format!("med_{id}"),
            MaterialKind::Font,
            StreamChoice::Disabled,
            StreamChoice::Disabled,
        )
    }));
}

pub(crate) fn add_text_scene(project: &mut ProjectEnvelope, mut text: Clip, duration: i64) {
    text.visual = Some(full_visual());
    project.project.sequences[0].tracks.extend([
        track(
            "trk_bg",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_bg", color(0, 0, 0), 0, duration)],
        ),
        track("trk_text", TrackKind::Visual, 1, vec![text]),
    ]);
}

pub(crate) fn key(id: &str, at: i64, value: f64, interpolation: Interpolation) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation,
    }
}

pub(super) fn unicode_fonts() -> BTreeMap<String, PathBuf> {
    let macos = [
        ("latin", "/System/Library/Fonts/SFNSMono.ttf"),
        ("emoji", "/System/Library/Fonts/Apple Symbols.ttf"),
        ("cjk", "/System/Library/Fonts/STHeiti Medium.ttc"),
        ("rtl", "/System/Library/Fonts/SFHebrew.ttf"),
        ("alt", "/System/Library/Fonts/Supplemental/Arial.ttf"),
    ];
    let linux = [
        (
            "latin",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        ),
        ("emoji", "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf"),
        (
            "cjk",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ),
        (
            "rtl",
            "/usr/share/fonts/truetype/noto/NotoSansHebrew-Regular.ttf",
        ),
        ("alt", "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf"),
    ];
    let paths = [macos, linux]
        .into_iter()
        .find(|candidate| candidate.iter().all(|(_, path)| Path::new(path).is_file()))
        .unwrap_or_else(|| panic!("CI must install the declared CJK, RTL, and emoji test fonts"));
    paths
        .into_iter()
        .map(|(id, path)| (format!("med_{id}"), PathBuf::from(path)))
        .collect()
}

pub(super) fn small_script_font() -> PathBuf {
    [
        "/System/Library/Fonts/Supplemental/NotoSansHatran-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansHatran-Regular.ttf",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
    .expect("CI must install a small Noto script font")
}

pub(super) fn lit_width(frame: &[u8]) -> usize {
    let xs: Vec<_> = frame
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, pixel)| pixel.iter().map(|value| usize::from(*value)).sum::<usize>() > 100)
        .map(|(index, _)| index % WIDTH as usize)
        .collect();
    xs.iter().max().unwrap_or(&0) - xs.iter().min().unwrap_or(&0) + 1
}
