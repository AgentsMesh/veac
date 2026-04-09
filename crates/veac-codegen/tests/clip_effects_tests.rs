/// Tests for new clip video effects: crop, blur, opacity, rotate, flip, vignette, grain, sharpen,
/// pan_x/pan_y, reverse, chromakey, normalize, loop, stabilize.
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

fn gen(clip: IrClip) -> String {
    let ir = IrProgram {
        outputs: vec![],
        project: make_project(),
        assets: vec![IrAsset {
            name: clip.asset_name.clone(),
            kind: IrAssetKind::Video,
            path: clip.asset_path.clone(),
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
    cmd.filter_graph.expect("should have filter_complex")
}

// --- Step 1: crop, blur, opacity, rotate, flip, vignette, grain, sharpen ---

#[test]
fn crop_generates_crop_filter() {
    let mut clip = make_clip("a", "a.mp4");
    clip.crop = Some("640x480+100+50".into());
    let fg = gen(clip);
    assert!(fg.contains("crop=640:480:100:50"));
}

#[test]
fn blur_generates_boxblur() {
    let mut clip = make_clip("a", "a.mp4");
    clip.blur = Some(10.0);
    let fg = gen(clip);
    assert!(fg.contains("boxblur=2:2"));
}

#[test]
fn opacity_generates_colorchannelmixer() {
    let mut clip = make_clip("a", "a.mp4");
    clip.opacity = Some(0.5);
    let fg = gen(clip);
    assert!(fg.contains("colorchannelmixer=aa=0.5"));
}

#[test]
fn full_opacity_no_filter() {
    let mut clip = make_clip("a", "a.mp4");
    clip.opacity = Some(1.0);
    let fg = gen(clip);
    assert!(!fg.contains("colorchannelmixer"));
}

#[test]
fn rotate_generates_rotate_filter() {
    let mut clip = make_clip("a", "a.mp4");
    clip.rotate = Some(90.0);
    let fg = gen(clip);
    assert!(fg.contains("rotate="));
}

#[test]
fn flip_horizontal() {
    let mut clip = make_clip("a", "a.mp4");
    clip.flip = Some("horizontal".into());
    let fg = gen(clip);
    assert!(fg.contains("hflip"));
}

#[test]
fn flip_vertical() {
    let mut clip = make_clip("a", "a.mp4");
    clip.flip = Some("vertical".into());
    let fg = gen(clip);
    assert!(fg.contains("vflip"));
}

#[test]
fn flip_both() {
    let mut clip = make_clip("a", "a.mp4");
    clip.flip = Some("both".into());
    let fg = gen(clip);
    assert!(fg.contains("hflip,vflip"));
}

#[test]
fn vignette_generates_vignette_filter() {
    let mut clip = make_clip("a", "a.mp4");
    clip.vignette = Some(0.5);
    let fg = gen(clip);
    assert!(fg.contains("vignette=angle="));
}

#[test]
fn grain_generates_noise_filter() {
    let mut clip = make_clip("a", "a.mp4");
    clip.grain = Some(0.3);
    let fg = gen(clip);
    assert!(fg.contains("noise=alls=30:allf=t"));
}

#[test]
fn sharpen_generates_unsharp_filter() {
    let mut clip = make_clip("a", "a.mp4");
    clip.sharpen = Some(1.5);
    let fg = gen(clip);
    assert!(fg.contains("unsharp=5:5:1.5:5:5:0"));
}

// --- Step 2: pan, reverse, chromakey ---

#[test]
fn reverse_generates_reverse_filters() {
    let mut clip = make_clip("a", "a.mp4");
    clip.reverse = Some(true);
    let fg = gen(clip);
    assert!(fg.contains("reverse"));
    assert!(fg.contains("areverse"));
}

#[test]
fn chromakey_green() {
    let mut clip = make_clip("a", "a.mp4");
    clip.chromakey = Some("green".into());
    let fg = gen(clip);
    assert!(fg.contains("chromakey=0x00FF00"));
}

#[test]
fn chromakey_hex_color() {
    let mut clip = make_clip("a", "a.mp4");
    clip.chromakey = Some("#00FF00".into());
    let fg = gen(clip);
    assert!(fg.contains("chromakey=0x00FF00"));
}

#[test]
fn zoompan_with_pan_offset() {
    let mut clip = make_clip("a", "a.mp4");
    clip.zoom = Some(2.0);
    clip.pan_x = Some(0.5);
    let fg = gen(clip);
    assert!(fg.contains("zoompan="));
    assert!(fg.contains("0.5"));
}

// --- Step 3: normalize, loop, stabilize ---

#[test]
fn normalize_generates_loudnorm() {
    let mut clip = make_clip("a", "a.mp4");
    clip.normalize = Some(true);
    let fg = gen(clip);
    assert!(fg.contains("loudnorm"));
}

#[test]
fn stabilize_generates_deshake() {
    let mut clip = make_clip("a", "a.mp4");
    clip.stabilize = Some(true);
    let fg = gen(clip);
    assert!(fg.contains("deshake"));
}

#[test]
fn loop_count_generates_concat() {
    let mut clip = make_clip("a", "a.mp4");
    clip.loop_count = Some(3);
    let fg = gen(clip);
    assert!(fg.contains("concat=n=3:v=1:a=1"));
}

// --- Effect chain order tests ---

#[test]
fn effect_chain_order_crop_before_scale() {
    let mut clip = make_clip("a", "a.mp4");
    clip.crop = Some("640x480+0+0".into());
    let fg = gen(clip);
    let crop_pos = fg.find("crop=").unwrap();
    let scale_pos = fg.find("scale=").unwrap();
    assert!(crop_pos < scale_pos, "crop should come before scale");
}

#[test]
fn effect_chain_order_blur_after_eq() {
    let mut clip = make_clip("a", "a.mp4");
    clip.brightness = Some(0.1);
    clip.blur = Some(5.0);
    let fg = gen(clip);
    let eq_pos = fg.find("eq=").unwrap();
    let blur_pos = fg.find("boxblur=").unwrap();
    assert!(eq_pos < blur_pos, "eq should come before blur");
}

// --- Semantic validation tests ---

#[test]
fn semantic_rejects_invalid_blur_range() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a { blur = 200.0 }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("blur"));
}

#[test]
fn semantic_rejects_invalid_opacity_range() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a { opacity = 1.5 }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("opacity"));
}

#[test]
fn semantic_rejects_invalid_flip_mode() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a { flip = "diagonal" }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("flip"));
}

#[test]
fn semantic_rejects_invalid_crop_format() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a { crop = "bad-format" }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("crop"));
}

#[test]
fn semantic_accepts_valid_clip_effects() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a {
                    crop = "640x480+0+0"
                    blur = 10.0
                    opacity = 0.8
                    rotate = 45.0
                    flip = "horizontal"
                    vignette = 0.5
                    grain = 0.2
                    sharpen = 1.0
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

#[test]
fn semantic_accepts_reverse_and_chromakey() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a {
                    reverse = true
                    chromakey = "green"
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

#[test]
fn semantic_accepts_normalize_loop_stabilize() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a {
                    normalize = true
                    loop = 2
                    stabilize = true
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

#[test]
fn semantic_rejects_loop_zero() {
    let src = r#"
        project "test" { resolution = "1920x1080" }
        asset a = video("a.mp4")
        timeline main {
            track video {
                clip a { loop = 0 }
            }
        }
    "#;
    let mut lexer = veac_lang::lexer::Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = veac_lang::parser::Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("."));
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("loop"));
}
