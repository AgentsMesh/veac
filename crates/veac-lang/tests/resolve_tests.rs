/// Tests for the MediaResolver: duration resolution priority, has_audio stamping, warnings.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use veac_lang::ir::*;
use veac_lang::resolve::*;

// ---------------------------------------------------------------------------
// Mock ProbeBackend
// ---------------------------------------------------------------------------

struct MockProbe {
    results: HashMap<PathBuf, MediaInfo>,
}

impl MockProbe {
    fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    fn add(&mut self, path: &str, duration: f64, has_audio: bool) {
        self.results.insert(
            PathBuf::from(path),
            MediaInfo {
                duration_secs: duration,
                has_video: true,
                has_audio,
                width: Some(1920),
                height: Some(1080),
                sample_rate: Some(44100),
            },
        );
    }
}

impl ProbeBackend for MockProbe {
    fn probe(&self, path: &Path) -> Result<MediaInfo, ProbeError> {
        self.results
            .get(path)
            .cloned()
            .ok_or_else(|| ProbeError::new(format!("mock: not found: {}", path.display())))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn make_ir(assets: Vec<IrAsset>, tracks: Vec<IrTrack>) -> IrProgram {
    IrProgram {
        project: make_project(),
        assets,
        timeline: IrTimeline {
            name: "main".into(),
            tracks,
        },
        outputs: vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests: Duration Resolution Priority
// ---------------------------------------------------------------------------

#[test]
fn resolved_duration_explicit_duration_wins() {
    let mut mock = MockProbe::new();
    mock.add("video.mp4", 120.0, true);

    let mut ir = make_ir(
        vec![IrAsset {
            name: "v".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("video.mp4"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Video,
            items: vec![IrTrackItem::Clip(IrClip {
                duration_sec: Some(5.0),
                from_sec: Some(10.0),
                to_sec: Some(30.0),
                ..make_clip("v", "video.mp4")
            })],
        }],
    );

    let resolver = MediaResolver::new(mock);
    let warnings = resolver.resolve(&mut ir);
    assert!(warnings.is_empty());

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        // Explicit duration (5.0) wins over to-from (20.0) and probe (120.0)
        assert_eq!(clip.resolved_duration, Some(5.0));
    } else {
        panic!("expected clip");
    }
}

#[test]
fn resolved_duration_from_to_wins_over_probe() {
    let mut mock = MockProbe::new();
    mock.add("video.mp4", 120.0, true);

    let mut ir = make_ir(
        vec![IrAsset {
            name: "v".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("video.mp4"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Video,
            items: vec![IrTrackItem::Clip(IrClip {
                from_sec: Some(10.0),
                to_sec: Some(25.0),
                ..make_clip("v", "video.mp4")
            })],
        }],
    );

    let resolver = MediaResolver::new(mock);
    resolver.resolve(&mut ir);

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        // to - from = 15.0 wins over probe duration (120.0)
        assert_eq!(clip.resolved_duration, Some(15.0));
    } else {
        panic!("expected clip");
    }
}

#[test]
fn resolved_duration_from_probe_when_no_trim() {
    let mut mock = MockProbe::new();
    mock.add("tts.mp3", 15.1, true);

    let mut ir = make_ir(
        vec![IrAsset {
            name: "narration".into(),
            kind: IrAssetKind::Audio,
            path: PathBuf::from("tts.mp3"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Audio,
            items: vec![IrTrackItem::Clip(IrClip {
                // No from/to/duration — should use probe duration
                ..make_clip("narration", "tts.mp3")
            })],
        }],
    );

    let resolver = MediaResolver::new(mock);
    let warnings = resolver.resolve(&mut ir);
    assert!(warnings.is_empty());

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        assert_eq!(clip.resolved_duration, Some(15.1));
    } else {
        panic!("expected clip");
    }
}

#[test]
fn resolved_duration_from_only_uses_probe_minus_from() {
    let mut mock = MockProbe::new();
    mock.add("video.mp4", 60.0, true);

    let mut ir = make_ir(
        vec![IrAsset {
            name: "v".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("video.mp4"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Video,
            items: vec![IrTrackItem::Clip(IrClip {
                from_sec: Some(20.0),
                // No to_sec — should use probe_duration - from = 40.0
                ..make_clip("v", "video.mp4")
            })],
        }],
    );

    let resolver = MediaResolver::new(mock);
    resolver.resolve(&mut ir);

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        assert_eq!(clip.resolved_duration, Some(40.0));
    } else {
        panic!("expected clip");
    }
}

#[test]
fn resolved_duration_with_speed_divides() {
    let mut mock = MockProbe::new();
    mock.add("video.mp4", 30.0, true);

    let mut ir = make_ir(
        vec![IrAsset {
            name: "v".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("video.mp4"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Video,
            items: vec![IrTrackItem::Clip(IrClip {
                speed: Some(2.0),
                ..make_clip("v", "video.mp4")
            })],
        }],
    );

    let resolver = MediaResolver::new(mock);
    resolver.resolve(&mut ir);

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        // 30.0 / 2.0 = 15.0
        assert_eq!(clip.resolved_duration, Some(15.0));
    } else {
        panic!("expected clip");
    }
}

// ---------------------------------------------------------------------------
// Tests: has_audio stamping
// ---------------------------------------------------------------------------

#[test]
fn has_audio_stamped_from_probe() {
    let mut mock = MockProbe::new();
    mock.add("video.mp4", 10.0, true);
    mock.add("image_seq.mp4", 10.0, false);

    let mut ir = make_ir(
        vec![
            IrAsset {
                name: "v".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("video.mp4"),
                media_info: None,
            },
            IrAsset {
                name: "noaudio".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("image_seq.mp4"),
                media_info: None,
            },
        ],
        vec![IrTrack {
            kind: IrTrackKind::Video,
            items: vec![
                IrTrackItem::Clip(make_clip("v", "video.mp4")),
                IrTrackItem::Clip(make_clip("noaudio", "image_seq.mp4")),
            ],
        }],
    );

    let resolver = MediaResolver::new(mock);
    resolver.resolve(&mut ir);

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        assert!(clip.has_audio);
    }
    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[1] {
        assert!(!clip.has_audio);
    }
}

// ---------------------------------------------------------------------------
// Tests: Warnings
// ---------------------------------------------------------------------------

#[test]
fn warning_on_probe_failure() {
    let mock = MockProbe::new(); // Empty — all probes fail

    let mut ir = make_ir(
        vec![IrAsset {
            name: "missing".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("nonexistent.mp4"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Video,
            items: vec![IrTrackItem::Clip(make_clip("missing", "nonexistent.mp4"))],
        }],
    );

    let resolver = MediaResolver::new(mock);
    let warnings = resolver.resolve(&mut ir);

    // Should have 2 warnings: one for asset probe failure, one for clip duration
    assert!(warnings.len() >= 1);
    assert!(warnings[0].message.contains("could not probe"));
}

#[test]
fn warning_on_no_duration_info() {
    let mock = MockProbe::new(); // Empty

    let mut ir = make_ir(
        vec![IrAsset {
            name: "x".into(),
            kind: IrAssetKind::Audio,
            path: PathBuf::from("x.mp3"),
            media_info: None,
        }],
        vec![IrTrack {
            kind: IrTrackKind::Audio,
            items: vec![IrTrackItem::Clip(IrClip {
                // No from/to/duration, no probe → should warn
                ..make_clip("x", "x.mp3")
            })],
        }],
    );

    let resolver = MediaResolver::new(mock);
    let warnings = resolver.resolve(&mut ir);

    let duration_warning = warnings.iter().any(|w| w.message.contains("no duration"));
    assert!(duration_warning, "expected duration warning, got: {warnings:?}");

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        assert_eq!(clip.resolved_duration, None);
    }
}

#[test]
fn media_info_populated_on_asset() {
    let mut mock = MockProbe::new();
    mock.add("video.mp4", 45.5, true);

    let mut ir = make_ir(
        vec![IrAsset {
            name: "v".into(),
            kind: IrAssetKind::Video,
            path: PathBuf::from("video.mp4"),
            media_info: None,
        }],
        vec![],
    );

    let resolver = MediaResolver::new(mock);
    resolver.resolve(&mut ir);

    let info = ir.assets[0].media_info.as_ref().expect("media_info should be populated");
    assert!((info.duration_secs - 45.5).abs() < 0.001);
    assert!(info.has_audio);
    assert_eq!(info.width, Some(1920));
}
