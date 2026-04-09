/// Tests for the refactored codegen: AudioRouting, skip_audio,
/// unified apply_audio_effects, and resolved_duration integration.
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

fn make_video_asset(name: &str, path: &str) -> IrAsset {
    IrAsset {
        name: name.into(),
        kind: IrAssetKind::Video,
        path: PathBuf::from(path),
        media_info: None,
    }
}

fn make_audio_asset(name: &str, path: &str) -> IrAsset {
    IrAsset {
        name: name.into(),
        kind: IrAssetKind::Audio,
        path: PathBuf::from(path),
        media_info: None,
    }
}

fn make_clip(name: &str, path: &str, from: f64, to: f64) -> IrClip {
    IrClip {
        asset_name: name.into(),
        asset_path: PathBuf::from(path),
        asset_kind: IrAssetKind::Video,
        from_sec: Some(from),
        to_sec: Some(to),
        ..Default::default()
    }
}

fn generate_filter(ir: &IrProgram) -> String {
    let cmd = veac_codegen::ffmpeg::generate(ir, Path::new("out.mp4"));
    cmd.filter_graph.unwrap_or_default()
}

// ---------------------------------------------------------------------------
// AudioRouting: separate audio track discards video audio
// ---------------------------------------------------------------------------

#[test]
fn separate_audio_track_discards_video_audio() {
    // Video track + Audio track → video audio should be discarded (silence + anullsink)
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            make_video_asset("intro", "./intro.mp4"),
            make_audio_asset("music", "./music.mp3"),
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("intro", "./intro.mp4", 0.0, 10.0))],
                },
                IrTrack {
                    kind: IrTrackKind::Audio,
                    items: vec![IrTrackItem::Clip(IrClip {
                        asset_name: "music".into(),
                        asset_path: PathBuf::from("./music.mp3"),
                        asset_kind: IrAssetKind::Audio,
                        volume: Some(0.8),
                        ..Default::default()
                    })],
                },
            ],
        },
    };

    let filter = generate_filter(&ir);

    // Should NOT contain atrim on 0:a (video audio extraction)
    // Should contain aevalsrc (silence) for video clip audio
    assert!(
        filter.contains("aevalsrc=0"),
        "expected silence for video audio when audio track exists. Got: {filter}"
    );
    // Should contain anullsink to consume the silence chain
    assert!(
        filter.contains("anullsink"),
        "expected anullsink for discarded video audio. Got: {filter}"
    );
    // Should contain atrim on 1:a (the audio track clip)
    assert!(
        filter.contains("[1:a]"),
        "expected audio track clip to reference input 1. Got: {filter}"
    );
}

#[test]
fn no_audio_track_keeps_video_audio() {
    // Video track only → video audio should be kept
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![make_video_asset("intro", "./intro.mp4")],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![IrTrackItem::Clip(make_clip("intro", "./intro.mp4", 0.0, 10.0))],
            }],
        },
    };

    let filter = generate_filter(&ir);

    // Should contain atrim on 0:a (video audio extraction, not silence)
    assert!(
        filter.contains("[0:a]"),
        "expected video audio extraction when no audio track. Got: {filter}"
    );
    // Should NOT contain anullsink
    assert!(
        !filter.contains("anullsink"),
        "should not sink audio when keeping it. Got: {filter}"
    );
}

// ---------------------------------------------------------------------------
// Unified apply_audio_effects: fade_out uses resolved_duration
// ---------------------------------------------------------------------------

#[test]
fn audio_track_fade_out_uses_resolved_duration() {
    // Audio clip with resolved_duration=15.0, fade_out=2.0
    // fade_out should start at 13.0 (not 0.0 or 8.0)
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            make_video_asset("intro", "./intro.mp4"),
            make_audio_asset("tts", "./tts.mp3"),
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("intro", "./intro.mp4", 0.0, 15.0))],
                },
                IrTrack {
                    kind: IrTrackKind::Audio,
                    items: vec![IrTrackItem::Clip(IrClip {
                        asset_name: "tts".into(),
                        asset_path: PathBuf::from("./tts.mp3"),
                        asset_kind: IrAssetKind::Audio,
                        resolved_duration: Some(15.0),
                        fade_out_sec: Some(2.0),
                        ..Default::default()
                    })],
                },
            ],
        },
    };

    let filter = generate_filter(&ir);

    // fade_out should start at 13.0 (= 15.0 - 2.0)
    assert!(
        filter.contains("afade=t=out:st=13:d=2"),
        "expected fade_out at st=13, d=2. Got: {filter}"
    );
}

#[test]
fn audio_track_fade_out_falls_back_to_estimate() {
    // Audio clip with from=5, to=20, no resolved_duration
    // Should estimate duration = 15.0, fade_out start = 13.0
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            make_video_asset("intro", "./intro.mp4"),
            make_audio_asset("bgm", "./bgm.mp3"),
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("intro", "./intro.mp4", 0.0, 15.0))],
                },
                IrTrack {
                    kind: IrTrackKind::Audio,
                    items: vec![IrTrackItem::Clip(IrClip {
                        asset_name: "bgm".into(),
                        asset_path: PathBuf::from("./bgm.mp3"),
                        asset_kind: IrAssetKind::Audio,
                        from_sec: Some(5.0),
                        to_sec: Some(20.0),
                        fade_out_sec: Some(2.0),
                        ..Default::default()
                    })],
                },
            ],
        },
    };

    let filter = generate_filter(&ir);

    // estimate: to - from = 15.0, fade_out start = 13.0
    assert!(
        filter.contains("afade=t=out:st=13:d=2"),
        "expected fade_out at st=13 from estimate. Got: {filter}"
    );
}

// ---------------------------------------------------------------------------
// Video clip audio also uses unified pipeline
// ---------------------------------------------------------------------------

#[test]
fn video_clip_audio_fade_out_correct() {
    // Video-only project: clip 0-10s, fade_out=1s → st=9
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![make_video_asset("v", "./v.mp4")],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![IrTrackItem::Clip(IrClip {
                    fade_out_sec: Some(1.0),
                    ..make_clip("v", "./v.mp4", 0.0, 10.0)
                })],
            }],
        },
    };

    let filter = generate_filter(&ir);

    // fade_out start = 10.0 - 1.0 = 9.0
    assert!(
        filter.contains("afade=t=out:st=9:d=1"),
        "expected video clip audio fade_out at st=9. Got: {filter}"
    );
}

// ---------------------------------------------------------------------------
// skip_audio: multiple clips with transitions
// ---------------------------------------------------------------------------

#[test]
fn skip_audio_with_transitions_uses_silence() {
    // Video track (2 clips + transition) + Audio track
    // Video clips should produce silence, not atrim on [0:a]
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            make_video_asset("a", "./a.mp4"),
            make_audio_asset("tts", "./tts.mp3"),
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![
                        IrTrackItem::Clip(make_clip("a", "./a.mp4", 0.0, 5.0)),
                        IrTrackItem::Transition(IrTransition {
                            kind: TransitionKind::Dissolve,
                            duration_sec: 0.5,
                        }),
                        IrTrackItem::Clip(make_clip("a", "./a.mp4", 10.0, 15.0)),
                    ],
                },
                IrTrack {
                    kind: IrTrackKind::Audio,
                    items: vec![IrTrackItem::Clip(IrClip {
                        asset_name: "tts".into(),
                        asset_path: PathBuf::from("./tts.mp3"),
                        asset_kind: IrAssetKind::Audio,
                        ..Default::default()
                    })],
                },
            ],
        },
    };

    let filter = generate_filter(&ir);

    // Count occurrences of aevalsrc (silence) — should be 2 (one per video clip)
    let silence_count = filter.matches("aevalsrc=0").count();
    assert!(
        silence_count >= 2,
        "expected at least 2 silence generators for skip_audio. Got {silence_count}. Filter: {filter}"
    );
    // Should NOT contain [0:a]atrim (no video audio extraction)
    assert!(
        !filter.contains("[0:a]atrim"),
        "should not extract video audio when skip_audio. Got: {filter}"
    );
}

// ---------------------------------------------------------------------------
// estimate_clip_duration prefers resolved_duration
// ---------------------------------------------------------------------------

#[test]
fn estimate_duration_prefers_resolved_over_fallback() {
    // Clip with no from/to/duration but has resolved_duration=42.0
    // Should NOT fall back to 10.0
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![make_video_asset("v", "./v.mp4")],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![IrTrack {
                kind: IrTrackKind::Video,
                items: vec![IrTrackItem::Clip(IrClip {
                    resolved_duration: Some(42.0),
                    fade_out_sec: Some(2.0),
                    ..IrClip {
                        asset_name: "v".into(),
                        asset_path: PathBuf::from("./v.mp4"),
                        asset_kind: IrAssetKind::Video,
                        ..Default::default()
                    }
                })],
            }],
        },
    };

    let filter = generate_filter(&ir);

    // fade_out start = 42.0 - 2.0 = 40.0 (not 10.0 - 2.0 = 8.0)
    assert!(
        filter.contains("afade=t=out:st=40:d=2"),
        "expected fade_out at st=40 using resolved_duration. Got: {filter}"
    );
}
