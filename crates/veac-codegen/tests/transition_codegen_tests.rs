/// Tests for transition codegen: xfade and acrossfade filter generation.
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

#[test]
fn transition_generates_xfade() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "a".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("a.mp4"),
            },
            IrAsset {
                name: "b".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("b.mp4"),
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![
                    IrTrackItem::Clip(make_clip("a", "a.mp4", Some(0.0), Some(10.0))),
                    IrTrackItem::Transition(IrTransition {
                        kind: TransitionKind::Fade,
                        duration_sec: 1.0,
                    }),
                    IrTrackItem::Clip(make_clip("b", "b.mp4", Some(0.0), Some(20.0))),
                ],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("xfade=transition=fade:duration=1"));
    assert!(fg.contains("acrossfade=d=1"));
}

#[test]
fn dissolve_transition() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "a".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("a.mp4"),
            },
            IrAsset {
                name: "b".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("b.mp4"),
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![
                    IrTrackItem::Clip(make_clip("a", "a.mp4", Some(0.0), Some(5.0))),
                    IrTrackItem::Transition(IrTransition {
                        kind: TransitionKind::Dissolve,
                        duration_sec: 2.0,
                    }),
                    IrTrackItem::Clip(make_clip("b", "b.mp4", Some(0.0), Some(10.0))),
                ],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("xfade=transition=dissolve:duration=2"));
}

#[test]
fn no_transition_uses_concat() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "a".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("a.mp4"),
            },
            IrAsset {
                name: "b".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("b.mp4"),
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![
                    IrTrackItem::Clip(make_clip("a", "a.mp4", Some(0.0), Some(5.0))),
                    IrTrackItem::Clip(make_clip("b", "b.mp4", Some(0.0), Some(10.0))),
                ],
            }],
        },
    };

    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("concat=n=2:v=1:a=1"));
    assert!(!fg.contains("xfade"));
}
