use std::path::Path;

use veac_lang::error::ErrorKind;
use veac_lang::ir::*;
use veac_lang::lexer::Lexer;
use veac_lang::parser::Parser;
use veac_lang::semantic::SemanticAnalyzer;

fn analyze(input: &str) -> Result<IrProgram, veac_lang::error::VeacError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    let analyzer = SemanticAnalyzer::new(Path::new("."));
    analyzer.analyze(&program)
}

#[test]
fn test_basic_program() {
    let ir = analyze(
        r#"
        project "test" {
            resolution = "1280x720"
            fps = 24
            format = "mp4"
        }
        asset clip1 = video("./test.mp4")
        timeline main {
            track video {
                clip clip1 {
                    from = 0s
                    to = 10s
                }
            }
        }
    "#,
    )
    .unwrap();

    assert_eq!(ir.project.width, 1280);
    assert_eq!(ir.project.height, 720);
    assert_eq!(ir.project.fps, 24);
    assert_eq!(ir.assets.len(), 1);
    assert_eq!(ir.timeline.tracks.len(), 1);
}

#[test]
fn test_variable_resolution() {
    let ir = analyze(
        r#"
        project "test" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
        asset clip1 = video("./test.mp4")
        let my_volume = 0.5
        timeline main {
            track video {
                clip clip1 {
                    volume = my_volume
                }
            }
        }
    "#,
    )
    .unwrap();

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        assert_eq!(clip.volume, Some(0.5));
    } else {
        panic!("expected clip");
    }
}

#[test]
fn test_undefined_asset_error() {
    let result = analyze(
        r#"
        project "test" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
        timeline main {
            track video {
                clip nonexistent {
                    from = 0s
                    to = 5s
                }
            }
        }
    "#,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind, ErrorKind::UndefinedAsset);
}

#[test]
fn test_invalid_time_range() {
    let result = analyze(
        r#"
        project "test" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
        asset clip1 = video("./test.mp4")
        timeline main {
            track video {
                clip clip1 {
                    from = 10s
                    to = 5s
                }
            }
        }
    "#,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind, ErrorKind::InvalidTimeRange);
}

#[test]
fn test_invalid_volume() {
    let result = analyze(
        r#"
        project "test" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
        asset clip1 = video("./test.mp4")
        timeline main {
            track video {
                clip clip1 {
                    volume = 1.5
                }
            }
        }
    "#,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind, ErrorKind::InvalidValue);
}

#[test]
fn test_missing_project() {
    let result = analyze(
        r#"
        asset clip1 = video("./test.mp4")
        timeline main {
            track video {
                clip clip1 {}
            }
        }
    "#,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind, ErrorKind::NoProject);
}

#[test]
fn test_text_overlay_resolution() {
    let ir = analyze(
        r#"
        project "test" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
        timeline main {
            track text {
                text "Hello" {
                    at = 2s
                    duration = 3s
                    font = "Helvetica"
                    size = 64
                    color = #FF0000
                    position = "bottom"
                }
            }
        }
    "#,
    )
    .unwrap();

    if let IrTrackItem::TextOverlay(t) = &ir.timeline.tracks[0].items[0] {
        assert_eq!(t.content, "Hello");
        assert_eq!(t.at_sec, 2.0);
        assert_eq!(t.duration_sec, 3.0);
        assert_eq!(t.font, "Helvetica");
        assert_eq!(t.size, 64);
        assert_eq!(t.color, "FF0000");
        assert_eq!(t.position, Position::Bottom);
    } else {
        panic!("expected text overlay");
    }
}

#[test]
fn test_time_conversions() {
    let ir = analyze(
        r#"
        project "test" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
        asset clip1 = video("./test.mp4")
        timeline main {
            track video {
                clip clip1 {
                    from = 500ms
                    to = 90f
                }
            }
        }
    "#,
    )
    .unwrap();

    if let IrTrackItem::Clip(clip) = &ir.timeline.tracks[0].items[0] {
        assert_eq!(clip.from_sec, Some(0.5));
        assert_eq!(clip.to_sec, Some(3.0)); // 90 frames / 30 fps = 3s
    }
}
