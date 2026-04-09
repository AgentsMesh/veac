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
