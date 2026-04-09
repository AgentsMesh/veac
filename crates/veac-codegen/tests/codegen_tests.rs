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

fn make_clip(name: &str, path: &str, from: Option<f64>, to: Option<f64>) -> IrClip {
    IrClip {
        asset_name: name.into(),
        asset_path: PathBuf::from(path),
        asset_kind: IrAssetKind::Video,
        from_sec: from,
        to_sec: to,
        ..Default::default()
    }
}

fn make_text_overlay() -> IrTextOverlay {
    IrTextOverlay {
        content: "Hello World".into(),
        at_sec: 3.0,
        duration_sec: 5.0,
        font: "Arial".into(),
        size: 48,
        color: "white".into(),
        position: Position::Center,
        fade_in_sec: None,
        fade_out_sec: None,
        resolved_font_path: None,
                background: None,
                background_padding: None,
    }
}

#[test]
fn single_clip_generates_valid_args() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "intro".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("./assets/intro.mp4"),
            media_info: None,
        }],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![IrTrackItem::Clip(make_clip(
                    "intro",
                    "./assets/intro.mp4",
                    Some(5.0),
                    Some(15.0),
                ))],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("output.mp4"));
    let args = cmd.to_args();

    assert!(args.contains(&"-i".to_string()));
    assert!(args.contains(&"./assets/intro.mp4".to_string()));
    assert!(args.contains(&"-filter_complex".to_string()));
    assert!(args.contains(&"-c:v".to_string()));
    assert!(args.contains(&"libx264".to_string()));
    assert!(args.contains(&"-preset".to_string()));
    assert!(args.contains(&"medium".to_string()));
    assert!(args.contains(&"output.mp4".to_string()));
}

#[test]
fn multiple_clips_use_concat() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "intro".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("./assets/intro.mp4"),
                media_info: None,
            },
            IrAsset {
                name: "scene1".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("./assets/scene1.mp4"),
                media_info: None,
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![
                    IrTrackItem::Clip(make_clip(
                        "intro",
                        "./assets/intro.mp4",
                        Some(5.0),
                        Some(15.0),
                    )),
                    IrTrackItem::Clip(make_clip(
                        "scene1",
                        "./assets/scene1.mp4",
                        Some(0.0),
                        Some(10.0),
                    )),
                ],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("output.mp4"));
    let args = cmd.to_args();

    // Should have two -i flags.
    let i_count = args.iter().filter(|a| *a == "-i").count();
    assert_eq!(i_count, 2);

    // Filter graph must include concat.
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("concat=n=2:v=1:a=1"));
}

#[test]
fn text_overlay_generates_drawtext() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "intro".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("./assets/intro.mp4"),
            media_info: None,
        }],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip(
                        "intro",
                        "./assets/intro.mp4",
                        None,
                        None,
                    ))],
                },
                IrTrack {
                    kind: IrTrackKind::Text,
                    items: vec![IrTrackItem::TextOverlay(make_text_overlay())],
                },
            ],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");

    assert!(fg.contains("drawtext=text='Hello World'"));
    assert!(fg.contains("fontsize=48"));
    assert!(fg.contains("fontcolor=white"));
    assert!(fg.contains("enable='between(t,3,8)'"));
}

#[test]
fn volume_adjustment_in_filter() {
    let mut clip = make_clip("intro", "./assets/intro.mp4", None, None);
    clip.volume = Some(0.5);

    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "intro".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("./assets/intro.mp4"),
            media_info: None,
        }],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![IrTrackItem::Clip(clip)],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");

    assert!(fg.contains("volume=0.5"));
}

#[test]
fn to_command_string_starts_with_ffmpeg() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "intro".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("./assets/intro.mp4"),
            media_info: None,
        }],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![IrTrackItem::Clip(make_clip(
                    "intro",
                    "./assets/intro.mp4",
                    None,
                    None,
                ))],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("output.mp4"));
    let s = cmd.to_command_string();
    assert!(s.starts_with("ffmpeg"));
    assert!(s.contains("output.mp4"));
}
