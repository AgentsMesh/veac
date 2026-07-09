/// Tests for Step 4-6: gap, freeze, text animation, pip, subtitle, letterbox, multi-output.
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

// --- Step 4: gap + freeze ---

#[test]
fn gap_generates_color_source_and_silence() {
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
                items: vec![
                    IrTrackItem::Clip(make_clip("a", "a.mp4")),
                    IrTrackItem::Gap(IrGap { duration_sec: 2.0 }),
                    IrTrackItem::Clip(make_clip("a", "a.mp4")),
                ],
            }],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("color=c=black:s=1920x1080:r=30:d=2"));
    assert!(fg.contains("aevalsrc=0:s=44100:d=2"));
    assert!(fg.contains("concat=n=3:v=1:a=1"));
}

#[test]
fn freeze_generates_trim_and_tpad() {
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
                items: vec![IrTrackItem::Freeze(IrFreeze {
                    asset_name: "a".into(),
                    asset_path: PathBuf::from("a.mp4"),
                    at_sec: 5.0,
                    duration_sec: 3.0,
                })],
            }],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("tpad=stop_mode=clone:stop_duration=3"));
    assert!(fg.contains("trim=start=5"));
}

#[test]
fn parse_gap_in_track() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a { from = 0s to = 5s }
                gap { duration = 2s }
                clip a { from = 5s to = 10s }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn parse_freeze_in_track() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                freeze a { at = 5s duration = 3s }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

// --- Step 5: text fade_in/fade_out + pip ---

#[test]
fn text_fade_in_out_generates_alpha() {
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
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("a", "a.mp4"))],
                },
                IrTrack {
                    kind: IrTrackKind::Text,
                    items: vec![IrTrackItem::TextOverlay(IrTextOverlay {
                        content: "Hello".into(),
                        at_sec: 1.0,
                        duration_sec: 5.0,
                        font: "Arial".into(),
                        size: 48,
                        color: "white".into(),
                        position: Position::Center,
                        fade_in_sec: Some(0.5),
                        fade_out_sec: Some(1.0),
                        resolved_font_path: None,
                background: None,
                background_padding: None,
                margin: None,
                    })],
                },
            ],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("alpha="));
    assert!(fg.contains("drawtext="));
}

#[test]
fn parse_text_with_fade() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video { clip a {} }
            track text {
                text "Hello" {
                    at = 1s
                    duration = 5s
                    fade_in = 0.5s
                    fade_out = 1s
                }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
    let ir = result.unwrap();
    // Find text overlay and verify fade fields
    for track in &ir.timeline.tracks {
        for item in &track.items {
            if let IrTrackItem::TextOverlay(t) = item {
                assert_eq!(t.fade_in_sec, Some(0.5));
                assert_eq!(t.fade_out_sec, Some(1.0));
            }
        }
    }
}

#[test]
fn pip_generates_overlay() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "main_vid".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("main.mp4"),
                media_info: None,
            },
            IrAsset {
                name: "cam".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("cam.mp4"),
                media_info: None,
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("main_vid", "main.mp4"))],
                },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::Pip(IrPip {
                        asset_name: "cam".into(),
                        asset_path: PathBuf::from("cam.mp4"),
                        from_sec: None,
                        to_sec: None,
                        at_sec: 0.0,
                        duration_sec: 10.0,
                        position: Position::BottomRight,
                        scale: 0.25,
                        zoom_in_sec: 0.0,
                        zoom_out_sec: 0.0,
                        fade_in_sec: 0.0,
                        fade_out_sec: 0.0,
                        margin_x: 0.0,
                        margin_y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    })],
                },
            ],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    // Should scale the pip to 25% of output dimensions
    assert!(fg.contains("scale=480:270"));
    assert!(fg.contains("overlay="));
    assert_eq!(cmd.inputs.len(), 2);
}

#[test]
fn pip_fade_generates_alpha_ramp() {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset {
                name: "main_vid".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("main.mp4"),
                media_info: None,
            },
            IrAsset {
                name: "roll".into(),
                kind: IrAssetKind::Video,
                path: PathBuf::from("roll.mp4"),
                media_info: None,
            },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("main_vid", "main.mp4"))],
                },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::Pip(IrPip {
                        asset_name: "roll".into(),
                        asset_path: PathBuf::from("roll.mp4"),
                        from_sec: None,
                        to_sec: None,
                        at_sec: 5.0,
                        duration_sec: 8.0,
                        position: Position::Center,
                        scale: 1.0,
                        zoom_in_sec: 0.0,
                        zoom_out_sec: 0.0,
                        fade_in_sec: 0.5,
                        fade_out_sec: 0.5,
                        margin_x: 0.0,
                        margin_y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    })],
                },
            ],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    // Alpha channel must be added, then ramped in at local 0 and out at duration-fade_out.
    assert!(fg.contains("format=yuva420p"), "pip fade needs an alpha channel: {fg}");
    assert!(fg.contains("fade=t=in:st=0:d=0.5:alpha=1"), "missing fade-in: {fg}");
    assert!(fg.contains("fade=t=out:st=7.5:d=0.5:alpha=1"), "missing fade-out: {fg}");
}

#[test]
fn pip_zoom_margin_insets_corner_but_fills_at_full() {
    // Output 1920x1080 (see make_project). A bottom-left zooming pip at scale 0.3 rests at
    // corner size 576x324; wmt=1920-576=1344, hmt=1080-324=756. With margin_x=48, margin_y=100 the
    // overlay x/y must interpolate 0→inset with the zoom (0 at full frame → no ghost).
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![
            IrAsset { name: "main_vid".into(), kind: IrAssetKind::Video, path: PathBuf::from("main.mp4"), media_info: None },
            IrAsset { name: "cam".into(), kind: IrAssetKind::Video, path: PathBuf::from("cam.mp4"), media_info: None },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack { kind: IrTrackKind::Video, items: vec![IrTrackItem::Clip(make_clip("main_vid", "main.mp4"))] },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::Pip(IrPip {
                        asset_name: "cam".into(),
                        asset_path: PathBuf::from("cam.mp4"),
                        from_sec: None,
                        to_sec: None,
                        at_sec: 2.0,
                        duration_sec: 6.0,
                        position: Position::BottomLeft,
                        scale: 0.3,
                        zoom_in_sec: 0.4,
                        zoom_out_sec: 0.4,
                        fade_in_sec: 0.0,
                        fade_out_sec: 0.0,
                        margin_x: 48.0,
                        margin_y: 100.0,
                        width: 0.0,
                        height: 0.0,
                    })],
                },
            ],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    // x: 48*(W-w)/1344 → 0 at full (w=W), 48 at corner (w=576). y: 656*(H-h)/756 → 0 at full, 656 at corner.
    assert!(fg.contains("48*(W-w)/1344"), "x should interpolate to a 48px left inset: {fg}");
    assert!(fg.contains("656*(H-h)/756"), "y should rest 656px down (1080-324-100): {fg}");
    assert!(fg.contains("eval=frame"), "animated overlay must re-eval position per frame: {fg}");
}

#[test]
fn pip_explicit_size_and_static_margin() {
    // A fade-only bubble: explicit square width/height (px) overrides the portrait scale, and the
    // bottom-left margin insets it to a constant resting position (no zoom → no per-frame eval).
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(), // 1920x1080
        assets: vec![
            IrAsset { name: "main_vid".into(), kind: IrAssetKind::Video, path: PathBuf::from("main.mp4"), media_info: None },
            IrAsset { name: "cam".into(), kind: IrAssetKind::Video, path: PathBuf::from("cam.mp4"), media_info: None },
        ],
        timeline: IrTimeline {
            name: "main".into(),
            tracks: vec![
                IrTrack { kind: IrTrackKind::Video, items: vec![IrTrackItem::Clip(make_clip("main_vid", "main.mp4"))] },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::Pip(IrPip {
                        asset_name: "cam".into(),
                        asset_path: PathBuf::from("cam.mp4"),
                        from_sec: None,
                        to_sec: None,
                        at_sec: 2.0,
                        duration_sec: 6.0,
                        position: Position::BottomLeft,
                        scale: 0.25, // ignored — width/height win
                        zoom_in_sec: 0.0,
                        zoom_out_sec: 0.0,
                        fade_in_sec: 0.4,
                        fade_out_sec: 0.4,
                        margin_x: 40.0,
                        margin_y: 100.0,
                        width: 300.0,
                        height: 300.0,
                    })],
                },
            ],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    // Square size from width/height, not scale (0.25*1920=480 would be wrong).
    assert!(fg.contains("scale=300:300"), "explicit px size should win over scale: {fg}");
    // Static bottom-left inset: x=margin_x=40, y=H-h-margin_y=1080-300-100=680.
    assert!(fg.contains("overlay=x=40:y=680"), "static margin should inset to a constant corner: {fg}");
}

#[test]
fn parse_pip_in_track() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset main_vid = video("main.mp4")
        asset cam = video("cam.mp4")
        timeline main {
            track video { clip main_vid {} }
            track overlay {
                pip cam {
                    at = 0s
                    duration = 10s
                    position = "bottom-right"
                    scale = 0.25
                }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

// --- Step 6: subtitle + letterbox + multi_output ---

#[test]
fn subtitle_generates_subtitles_filter() {
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
            tracks: vec![
                IrTrack {
                    kind: IrTrackKind::Video,
                    items: vec![IrTrackItem::Clip(make_clip("a", "a.mp4"))],
                },
                IrTrack {
                    kind: IrTrackKind::Overlay,
                    items: vec![IrTrackItem::Subtitle(IrSubtitle {
                        path: PathBuf::from("subs.srt"),
                    })],
                },
            ],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("subtitles=filename='subs.srt'"));
}

#[test]
fn parse_subtitle_in_track() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video { clip a {} }
            track overlay {
                subtitle "subs.srt" {}
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn letterbox_generates_pad_filter() {
    let mut project = make_project();
    project.fit = FitMode::Letterbox;
    let ir = IrProgram {
        outputs: vec![],
        project,
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
                items: vec![IrTrackItem::Clip(make_clip("a", "a.mp4"))],
            }],
        },
    };
    let cmd = veac_codegen::ffmpeg::generate(&ir, Path::new("out.mp4"));
    let fg = cmd.filter_graph.expect("should have filter_complex");
    assert!(fg.contains("pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=black"));
}

#[test]
fn parse_letterbox_fit_mode() {
    let src = r#"
        project "test" {
            resolution = "1920x1080"
            fit = "letterbox"
        }
        asset a = video("a.mp4")
        timeline main {
            track video { clip a {} }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().project.fit, FitMode::Letterbox);
}

#[test]
fn multi_output_generates_multiple_commands() {
    let ir = IrProgram {
        outputs: vec![
            IrOutputConfig {
                path: PathBuf::from("out_720.mp4"),
                width: Some(1280),
                height: Some(720),
                format: None,
                codec: None,
                quality: Some(Quality::Medium),
            },
            IrOutputConfig {
                path: PathBuf::from("out_1080.mp4"),
                width: None,
                height: None,
                format: None,
                codec: None,
                quality: Some(Quality::High),
            },
        ],
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
                items: vec![IrTrackItem::Clip(make_clip("a", "a.mp4"))],
            }],
        },
    };
    let cmds = veac_codegen::ffmpeg::generate_all(&ir, Path::new("default.mp4"));
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].output_path, PathBuf::from("out_720.mp4"));
    assert_eq!(cmds[1].output_path, PathBuf::from("out_1080.mp4"));
    // First output should use 1280x720 resolution
    let args0 = cmds[0].to_args();
    assert!(args0.contains(&"1280x720".to_string()));
}

#[test]
fn parse_output_declaration() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video { clip a {} }
        }
        output "out_720.mp4" {
            resolution = "1280x720"
            quality = "medium"
        }
        output "out_1080.mp4" {
            quality = "high"
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    assert_eq!(program.outputs.len(), 2);
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
    let ir = result.unwrap();
    assert_eq!(ir.outputs.len(), 2);
}

#[test]
fn no_outputs_falls_back_to_single() {
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
                items: vec![IrTrackItem::Clip(make_clip("a", "a.mp4"))],
            }],
        },
    };
    let cmds = veac_codegen::ffmpeg::generate_all(&ir, Path::new("default.mp4"));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].output_path, PathBuf::from("default.mp4"));
}
