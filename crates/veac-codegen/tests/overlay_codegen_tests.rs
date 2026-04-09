/// Tests for overlay codegen: image overlay and scale filter generation.
use std::path::{Path, PathBuf};

use veac_lang::ir::*;

fn make_project() -> IrProject {
    IrProject {
        name: "test".into(),
        width: 1920,
        height: 1080,
        fps: 30,
        format: OutputFormat::Mp4,
        codec: Codec::H264,
        quality: Quality::Medium,
        fit: FitMode::Fill,
    }
}

fn make_clip(name: &str, path: &str) -> IrClip {
    IrClip {
        asset_name: name.into(),
        asset_path: PathBuf::from(path),
        asset_kind: IrAssetKind::Video,
        ..Default::default()
    }
}

#[test]
fn image_overlay_generates_overlay_filter() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "intro".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("intro.mp4"),
                media_info: None,
            },
            IrAsset {
                name: "logo".into(),
                kind: IrAssetKind::Image,
                path: PathBuf::from("logo.png"),
                media_info: None,
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("intro", "intro.mp4"))],
                },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::ImageOverlay(IrImageOverlay {
                        asset_name: "logo".into(),
                        asset_path: PathBuf::from("logo.png"),
                        at_sec: 0.0,
                        duration_sec: 30.0,
                        position: Position::TopRight,
                        scale: Some(0.1),
                        opacity: Some(0.8),
                    })],
                },
            ],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");

    // Should have scale for the image
    assert!(fg.contains("scale=iw*0.1:ih*0.1"));
    // Should have opacity adjustment
    assert!(fg.contains("colorchannelmixer=aa=0.8"));
    // Should have overlay filter with position
    assert!(fg.contains("overlay="));
    assert!(fg.contains("enable='between(t,0,30)'"));
    // Should have 2 inputs
    assert_eq!(cmd.inputs.len(), 2);
}

#[test]
fn image_overlay_without_scale_or_opacity() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "intro".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("intro.mp4"),
                media_info: None,
            },
            IrAsset {
                name: "watermark".into(),
                kind: IrAssetKind::Image,
                path: PathBuf::from("wm.png"),
                media_info: None,
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("intro", "intro.mp4"))],
                },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::ImageOverlay(IrImageOverlay {
                        asset_name: "watermark".into(),
                        asset_path: PathBuf::from("wm.png"),
                        at_sec: 5.0,
                        duration_sec: 10.0,
                        position: Position::BottomRight,
                        scale: None,
                        opacity: None,
                    })],
                },
            ],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");

    // No image-specific scale (iw*) or opacity filters
    assert!(!fg.contains("scale=iw*"));
    assert!(!fg.contains("colorchannelmixer"));
    // But should still have overlay
    assert!(fg.contains("overlay="));
    assert!(fg.contains("enable='between(t,5,15)'"));
}
