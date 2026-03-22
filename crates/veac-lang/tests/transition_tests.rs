use veac_lang::ast::TrackItem;
/// Tests for transition syntax parsing and semantic validation.
use veac_lang::lexer::Lexer;
use veac_lang::parser::Parser;

fn parse(source: &str) -> veac_lang::ast::Program {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("lexer error");
    let mut parser = Parser::new(tokens);
    parser.parse().expect("parser error")
}

#[test]
fn parse_transition_in_video_track() {
    let src = r#"
        project "test" { resolution = "1920x1080" fps = 30 }
        asset intro = video("intro.mp4")
        asset scene1 = video("scene1.mp4")
        timeline main {
            track video {
                clip intro { from = 0s to = 10s }
                transition { type = "fade" duration = 1s }
                clip scene1 { from = 0s to = 20s }
            }
        }
    "#;

    let program = parse(src);
    let track = &program.timelines[0].tracks[0];
    assert_eq!(track.items.len(), 3);
    assert!(matches!(track.items[0], TrackItem::Clip(_)));
    assert!(matches!(track.items[1], TrackItem::Transition(_)));
    assert!(matches!(track.items[2], TrackItem::Clip(_)));
}

#[test]
fn parse_transition_attributes() {
    let src = r#"
        project "test" {}
        asset a = video("a.mp4")
        asset b = video("b.mp4")
        timeline main {
            track video {
                clip a {}
                transition { type = "dissolve" duration = 2s }
                clip b {}
            }
        }
    "#;

    let program = parse(src);
    let track = &program.timelines[0].tracks[0];
    if let TrackItem::Transition(t) = &track.items[1] {
        assert_eq!(t.attributes.len(), 2);
        assert_eq!(t.attributes[0].key, "type");
        assert_eq!(t.attributes[1].key, "duration");
    } else {
        panic!("expected transition");
    }
}

#[test]
fn semantic_resolve_transition() {
    let src = r#"
        project "test" { resolution = "1920x1080" fps = 30 }
        asset intro = video("intro.mp4")
        asset scene1 = video("scene1.mp4")
        timeline main {
            track video {
                clip intro { from = 0s to = 10s }
                transition { type = "fade" duration = 1s }
                clip scene1 { from = 0s to = 20s }
            }
        }
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    // Use /tmp as base_dir so absolute paths won't fail file-not-found
    // (semantic analyzer doesn't check asset files in this flow)
    let analyzer = veac_lang::semantic::SemanticAnalyzer::new(std::path::Path::new("/tmp"));
    let ir = analyzer.analyze(&program).unwrap();

    let track = &ir.timeline.tracks[0];
    assert_eq!(track.items.len(), 3);
    assert!(matches!(
        track.items[1],
        veac_lang::ir::IrTrackItem::Transition(_)
    ));

    if let veac_lang::ir::IrTrackItem::Transition(t) = &track.items[1] {
        assert_eq!(t.kind, veac_lang::ir::TransitionKind::Fade);
        assert_eq!(t.duration_sec, 1.0);
    }
}

#[test]
fn semantic_reject_invalid_transition_type() {
    let src = r#"
        project "test" { resolution = "1920x1080" fps = 30 }
        asset a = video("a.mp4")
        asset b = video("b.mp4")
        timeline main {
            track video {
                clip a { from = 0s to = 10s }
                transition { type = "invalid_type" duration = 1s }
                clip b { from = 0s to = 20s }
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
    let err = result.unwrap_err();
    assert!(err.message.contains("unknown transition type"));
}
