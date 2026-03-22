use veac_lang::ast::*;
use veac_lang::lexer::Lexer;
use veac_lang::parser::Parser;

fn parse(input: &str) -> Program {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("lexer error");
    let mut parser = Parser::new(tokens);
    parser.parse().expect("parser error")
}

#[test]
fn test_parse_project() {
    let prog = parse(
        r#"
        project "my-video" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }
    "#,
    );
    let proj = prog.project.unwrap();
    assert_eq!(proj.name, "my-video");
    assert_eq!(proj.attributes.len(), 3);
    assert_eq!(proj.attributes[0].key, "resolution");
}

#[test]
fn test_parse_asset() {
    let prog = parse(
        r#"
        asset intro = video("./assets/intro.mp4")
        asset bgm = audio("./assets/bgm.mp3")
        asset logo = image("./assets/logo.png")
    "#,
    );
    assert_eq!(prog.assets.len(), 3);
    assert_eq!(prog.assets[0].name, "intro");
    assert_eq!(prog.assets[0].kind, AssetKind::Video);
    assert_eq!(prog.assets[1].kind, AssetKind::Audio);
    assert_eq!(prog.assets[2].kind, AssetKind::Image);
}

#[test]
fn test_parse_let() {
    let prog = parse("let fade_duration = 1s\nlet default_volume = 0.8");
    assert_eq!(prog.variables.len(), 2);
    assert_eq!(prog.variables[0].name, "fade_duration");
    matches!(prog.variables[0].value, Expression::TimeSec(1.0));
}

#[test]
fn test_parse_timeline() {
    let prog = parse(
        r#"
        timeline main {
            track video {
                clip intro {
                    from = 0s
                    to = 10s
                }
            }
            track audio {
                clip bgm {
                    volume = 0.8
                }
            }
        }
    "#,
    );
    assert_eq!(prog.timelines.len(), 1);
    let tl = &prog.timelines[0];
    assert_eq!(tl.name, "main");
    assert_eq!(tl.tracks.len(), 2);
    assert_eq!(tl.tracks[0].kind, TrackKind::Video);
    assert_eq!(tl.tracks[1].kind, TrackKind::Audio);
}

#[test]
fn test_parse_text_overlay() {
    let prog = parse(
        r#"
        timeline main {
            track text {
                text "Hello World" {
                    at = 3s
                    duration = 5s
                    font = "Arial"
                    size = 48
                    color = #FFFFFF
                    position = "center"
                }
            }
        }
    "#,
    );
    let tl = &prog.timelines[0];
    let item = &tl.tracks[0].items[0];
    match item {
        TrackItem::TextOverlay(t) => {
            assert_eq!(t.content, "Hello World");
            assert_eq!(t.attributes.len(), 6);
        }
        _ => panic!("expected TextOverlay"),
    }
}

#[test]
fn test_parse_full_program() {
    let prog = parse(
        r#"
        project "demo" {
            resolution = "1920x1080"
            fps = 30
            format = "mp4"
        }

        asset intro = video("./assets/intro.mp4")
        asset bgm = audio("./assets/bgm.mp3")

        let default_volume = 0.8

        timeline main {
            track video {
                clip intro {
                    from = 0s
                    to = 10s
                }
            }
            track audio {
                clip bgm {
                    volume = default_volume
                }
            }
            track text {
                text "Welcome" {
                    at = 1s
                    duration = 3s
                    size = 48
                    color = #FFFFFF
                    position = "center"
                }
            }
        }
    "#,
    );

    assert!(prog.project.is_some());
    assert_eq!(prog.assets.len(), 2);
    assert_eq!(prog.variables.len(), 1);
    assert_eq!(prog.timelines.len(), 1);
    assert_eq!(prog.timelines[0].tracks.len(), 3);
}

#[test]
fn test_parse_error_unexpected_token() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_err());
}
