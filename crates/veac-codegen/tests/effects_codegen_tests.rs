/// Tests for effects codegen: speed, eq (color grading), and audio fades.
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
        from_sec: Some(0.0),
        to_sec: Some(10.0),
        ..Default::default()
    }
}

#[test]
fn speed_generates_setpts_and_atempo() {
    let mut clip = make_clip("a", "a.mp4");
    clip.speed = Some(2.0);

    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "a".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("a.mp4"),
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

    assert!(fg.contains("setpts=PTS/2"));
    assert!(fg.contains("atempo=2"));
}

#[test]
fn color_grading_generates_eq() {
    let mut clip = make_clip("a", "a.mp4");
    clip.brightness = Some(0.1);
    clip.contrast = Some(1.2);
    clip.saturation = Some(1.5);

    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "a".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("a.mp4"),
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

    assert!(fg.contains("eq=brightness=0.1:contrast=1.2:saturation=1.5"));
}

#[test]
fn audio_fade_generates_afade() {
    let mut clip = make_clip("bgm", "bgm.mp3");
    clip.asset_kind = IrAssetKind::Audio;
    clip.fade_in_sec = Some(2.0);
    clip.fade_out_sec = Some(3.0);

    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "bgm".into(),
            kind: IrAssetKind::Audio,
            path: PathBuf::from("bgm.mp3"),
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

    assert!(fg.contains("afade=t=in:st=0:d=2"));
    assert!(fg.contains("afade=t=out:"));
}

#[test]
fn partial_color_grading_only_brightness() {
    let mut clip = make_clip("a", "a.mp4");
    clip.brightness = Some(-0.2);

    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "a".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("a.mp4"),
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

    assert!(fg.contains("eq=brightness=-0.2"));
    assert!(!fg.contains("contrast"));
    assert!(!fg.contains("saturation"));
}

#[test]
fn no_effects_no_extra_filters() {
    let clip = make_clip("a", "a.mp4");

    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: "a".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("a.mp4"),
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

    assert!(!fg.contains("setpts=PTS/"));
    assert!(!fg.contains("atempo"));
    assert!(!fg.contains("eq="));
    assert!(!fg.contains("afade"));
}
