use veac_lang::ast::TrackItem;
/// Tests for image overlay parsing, include syntax, and semantic validation.
use veac_lang::lexer::Lexer;
use veac_lang::parser::Parser;

fn parse(source: &str) -> veac_lang::ast::Program {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("lexer error");
    let mut parser = Parser::new(tokens);
    parser.parse().expect("parser error")
}

#[test]
fn parse_image_overlay_in_track() {
    let src = r#"
        project "test" {}
        asset logo = image("logo.png")
        timeline main {
            track overlay {
                image logo {
                    at = 0s
                    duration = 30s
                    position = "top-right"
                    scale = 0.1
                    opacity = 0.8
                }
            }
        }
    "#;

    let program = parse(src);
    let track = &program.timelines[0].tracks[0];
    assert_eq!(track.items.len(), 1);
    assert!(matches!(track.items[0], TrackItem::ImageOverlay(_)));

    if let TrackItem::ImageOverlay(img) = &track.items[0] {
        assert_eq!(img.asset_ref, "logo");
        assert_eq!(img.attributes.len(), 5);
    }
}

#[test]
fn parse_include_statement() {
    let src = r#"
        include "./templates/intro.veac"
        project "test" {}
        timeline main {
            track video {
            }
        }
    "#;

    let program = parse(src);
    assert_eq!(program.includes.len(), 1);
    assert_eq!(
        program.includes[0].path.to_str().unwrap(),
        "./templates/intro.veac"
    );
}

#[test]
fn parse_multiple_includes() {
    let src = r#"
        include "./a.veac"
        include "./b.veac"
        project "test" {}
        timeline main { track video {} }
    "#;

    let program = parse(src);
    assert_eq!(program.includes.len(), 2);
}

#[test]
fn parse_overlay_track_kind() {
    let src = r#"
        project "test" {}
        timeline main {
            track overlay {
            }
        }
    "#;

    let program = parse(src);
    assert_eq!(
        program.timelines[0].tracks[0].kind,
        veac_lang::ast::TrackKind::Overlay
    );
}

#[test]
fn semantic_resolve_image_overlay() {
    let src = r#"
        project "test" { resolution = "1920x1080" fps = 30 }
        asset logo = image("logo.png")
        asset intro = video("intro.mp4")
        timeline main {
            track video {
                clip intro { from = 0s to = 10s }
            }
            track overlay {
                image logo {
                    at = 0s
                    duration = 10s
                    position = "top-right"
                    scale = 0.2
                    opacity = 0.7
                }
            }
        }
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("/tmp"));
    let ir = analyzer.analyze(&program).unwrap();

    // Find the overlay track
    let overlay_track = ir
        .timeline
        .tracks
        .iter()
        .find(|t| t.kind == veac_lang::ir::IrTrackKind::Overlay)
        .expect("should have overlay track");

    assert_eq!(overlay_track.items.len(), 1);
    if let veac_lang::ir::IrTrackItem::ImageOverlay(ov) = &overlay_track.items[0] {
        assert_eq!(ov.asset_name, "logo");
        assert_eq!(ov.at_sec, 0.0);
        assert_eq!(ov.duration_sec, 10.0);
        assert_eq!(ov.position, veac_lang::ir::Position::TopRight);
        assert_eq!(ov.scale, Some(0.2));
        assert_eq!(ov.opacity, Some(0.7));
    } else {
        panic!("expected image overlay");
    }
}

#[test]
fn semantic_reject_invalid_opacity() {
    let src = r#"
        project "test" { resolution = "1920x1080" fps = 30 }
        asset logo = image("logo.png")
        asset intro = video("intro.mp4")
        timeline main {
            track video { clip intro {} }
            track overlay {
                image logo { opacity = 1.5 }
            }
        }
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("/tmp"));
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("opacity"));
}

#[test]
fn lexer_recognizes_include_keyword() {
    let src = "include";
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // Include + Eof
    assert_eq!(tokens[0].kind, veac_lang::token::TokenKind::Include);
}
