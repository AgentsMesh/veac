/// Tests for the overlay "card" primitives: fit, rounded corners, drop shadow, image fade,
/// explicit box/size, x/y positioning, text shadow/outline, and the image-clip duration fix.
/// Each asserts on the generated `-filter_complex` string (mutation-verified: removing the
/// feature's codegen turns the matching assertion red).
use std::path::{Path, PathBuf};

use veac_lang::ir::*;

fn project() -> IrProject {
    IrProject {
        name: "t".into(),
        width: 1080,
        height: 1920,
        fps: 30,
        format: OutputFormat::Mp4,
        codec: Codec::H264,
        quality: Quality::Medium,
        fit: FitMode::Fill,
    }
}

fn asset(name: &str, kind: IrAssetKind, path: &str) -> IrAsset {
    IrAsset {
        name: name.into(),
        kind,
        path: PathBuf::from(path),
        media_info: None,
    }
}

fn clip(name: &str) -> IrClip {
    IrClip {
        asset_name: name.into(),
        asset_path: PathBuf::from(format!("{name}.mp4")),
        asset_kind: IrAssetKind::Video,
        to_sec: Some(3.0),
        has_audio: true,
        ..Default::default()
    }
}

/// Build a program with a base video track + one overlay track, return the filter_complex.
fn fg(assets: Vec<IrAsset>, base: IrTrackItem, overlay: Vec<IrTrackItem>) -> String {
    let ir = IrProgram {
        outputs: vec![],
        project: project(),
        assets,
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![base],
                },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: overlay,
                },
            ],
        },
    };
    veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"))
        .filter_graph
        .expect("filter_complex")
}

fn pip(style: CardStyle) -> IrPip {
    IrPip {
        asset_name: "w".into(),
        asset_path: PathBuf::from("w.mp4"),
        at_sec: 0.0,
        duration_sec: 3.0,
        width: 740.0,
        height: 775.0,
        style,
        ..Default::default()
    }
}

fn base_and_widget() -> (Vec<IrAsset>, IrTrackItem) {
    (
        vec![
            asset("bg", IrAssetKind::Video, "bg.mp4"),
            asset("w", IrAssetKind::Video, "w.mp4"),
        ],
        IrTrackItem::Clip(clip("bg")),
    )
}

#[test]
fn pip_radius_rounds_corners_preserving_rgb() {
    let (assets, base) = base_and_widget();
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::Pip(pip(CardStyle {
            radius: Some(64),
            ..Default::default()
        }))],
    );
    // geq copies RGB and only shapes alpha — the corner arc test with the radius must be present.
    assert!(g.contains("geq="), "expected geq for rounded corners: {g}");
    assert!(
        g.contains("r='r(X\\,Y)'"),
        "RGB must be copied verbatim: {g}"
    );
    assert!(
        g.contains("hypot(") && g.contains("64"),
        "corner arc test with radius: {g}"
    );
}

#[test]
fn pip_shadow_splits_blurs_and_double_overlays() {
    let (assets, base) = base_and_widget();
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::Pip(pip(CardStyle {
            shadow: Some(Shadow::default()),
            ..Default::default()
        }))],
    );
    assert!(g.contains("split"), "shadow needs a split: {g}");
    assert!(g.contains("boxblur"), "shadow must be blurred: {g}");
    assert!(
        g.contains("alpha_radius"),
        "shadow blurs the alpha edge: {g}"
    );
    assert_eq!(
        g.matches("overlay=").count(),
        2,
        "shadow then card = two overlays: {g}"
    );
}

#[test]
fn pip_fit_cover_uses_increase_and_crop() {
    let (assets, base) = base_and_widget();
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::Pip(pip(CardStyle {
            fit: Some(FitMode::Crop),
            ..Default::default()
        }))],
    );
    assert!(g.contains("force_original_aspect_ratio=increase"), "{g}");
    assert!(g.contains("crop=740:775"), "cover crops to the box: {g}");
}

#[test]
fn pip_fit_contain_preserves_aspect() {
    let (assets, base) = base_and_widget();
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::Pip(pip(CardStyle {
            fit: Some(FitMode::Letterbox),
            ..Default::default()
        }))],
    );
    assert!(g.contains("force_original_aspect_ratio=decrease"), "{g}");
}

#[test]
fn pip_default_fit_is_plain_scale() {
    let (assets, base) = base_and_widget();
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::Pip(pip(CardStyle::default()))],
    );
    assert!(
        g.contains("scale=740:775"),
        "fill fit is a plain stretch: {g}"
    );
    assert!(
        !g.contains("force_original_aspect_ratio"),
        "no aspect lock by default: {g}"
    );
}

fn image(name: &str) -> IrImageOverlay {
    IrImageOverlay {
        asset_name: name.into(),
        asset_path: PathBuf::from(format!("{name}.png")),
        at_sec: 0.0,
        duration_sec: 3.0,
        ..Default::default()
    }
}

fn base_and_image(name: &str) -> (Vec<IrAsset>, IrTrackItem) {
    (
        vec![
            asset("bg", IrAssetKind::Video, "bg.mp4"),
            asset(name, IrAssetKind::Image, &format!("{name}.png")),
        ],
        IrTrackItem::Clip(clip("bg")),
    )
}

#[test]
fn image_fade_loops_still_and_ramps_alpha() {
    let (assets, base) = base_and_image("cap");
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::ImageOverlay(IrImageOverlay {
            fade_in_sec: Some(0.3),
            fade_out_sec: Some(0.3),
            ..image("cap")
        })],
    );
    assert!(
        g.contains("loop=loop=-1"),
        "a still must be looped to gain frames to fade: {g}"
    );
    assert!(
        g.contains("fade=t=in") && g.contains("alpha=1"),
        "alpha ramp: {g}"
    );
}

#[test]
fn image_explicit_box_scales_to_size() {
    let (assets, base) = base_and_image("cap");
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::ImageOverlay(IrImageOverlay {
            width: Some(600.0),
            height: Some(160.0),
            ..image("cap")
        })],
    );
    assert!(
        g.contains("scale=600:160"),
        "explicit box overrides scale: {g}"
    );
}

#[test]
fn image_xy_positions_exactly() {
    let (assets, base) = base_and_image("cap");
    let g = fg(
        assets,
        base,
        vec![IrTrackItem::ImageOverlay(IrImageOverlay {
            x: Some(120.0),
            y: Some(240.0),
            ..image("cap")
        })],
    );
    assert!(
        g.contains("overlay=x=120:y=240"),
        "explicit pixel position: {g}"
    );
}

#[test]
fn image_clip_on_main_track_loops_to_duration() {
    // The bug fix: a still on the video track must honor `duration` via a loop, not collapse.
    let ir_assets = vec![asset("bgimg", IrAssetKind::Image, "bg.png")];
    let base = IrTrackItem::Clip(IrClip {
        asset_name: "bgimg".into(),
        asset_path: PathBuf::from("bg.png"),
        asset_kind: IrAssetKind::Image,
        duration_sec: Some(4.0),
        has_audio: false,
        ..Default::default()
    });
    let g = fg(ir_assets, base, vec![]);
    assert!(
        g.contains("loop=loop=-1"),
        "image clip loops to its duration: {g}"
    );
    assert!(
        g.contains("trim=duration=4"),
        "looped to the requested duration: {g}"
    );
}

fn text(content: &str) -> IrTextOverlay {
    IrTextOverlay {
        content: content.into(),
        at_sec: 0.0,
        duration_sec: 3.0,
        font: "Arial".into(),
        size: 60,
        color: "white".into(),
        ..Default::default()
    }
}

#[test]
fn text_shadow_and_outline_reach_drawtext() {
    let assets = vec![asset("bg", IrAssetKind::Video, "bg.mp4")];
    let base = IrTrackItem::Clip(clip("bg"));
    let g = {
        let ir = IrProgram {
            outputs: vec![],
            project: project(),
            assets,
            timeline: IrTimeline {
                name: "main".into(),
                tracks: vec![
                    IrTrack {
                        kind: IrTrackKind::Video,
                        items: vec![base],
                    },
                    IrTrack {
                        kind: IrTrackKind::Text,
                        items: vec![IrTrackItem::TextOverlay(IrTextOverlay {
                            shadow: Some(Shadow {
                                blur: 0.0,
                                opacity: 1.0,
                                dx: 3.0,
                                dy: 3.0,
                                color: "black".into(),
                            }),
                            outline: Some(Outline {
                                width: 4,
                                color: "0x101010".into(),
                            }),
                            ..text("Hi")
                        })],
                    },
                ],
            },
        };
        veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"))
            .filter_graph
            .expect("fg")
    };
    assert!(
        g.contains("shadowx=3") && g.contains("shadowy=3"),
        "drawtext shadow: {g}"
    );
    assert!(
        g.contains("borderw=4") && g.contains("bordercolor=0x101010"),
        "drawtext outline: {g}"
    );
}

#[test]
fn image_scroll_animates_position_linearly() {
    let (assets, base) = base_and_image("river");
    let g = fg(assets, base, vec![IrTrackItem::ImageOverlay(IrImageOverlay {
        width: Some(1080.0),
        height: Some(5000.0),
        y: Some(0.0),
        scroll_y: -3000.0,
        ..image("river")
    })]);
    assert!(g.contains("eval=frame"), "scroll uses per-frame animated overlay: {g}");
    assert!(g.contains("(-3000)*(t-0)"), "linear travel expression over the window: {g}");
}
