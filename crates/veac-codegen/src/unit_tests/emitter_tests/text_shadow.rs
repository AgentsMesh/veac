use super::support::{ass_script, bindings, emit_video_command, resolved, text_fixture};

#[test]
fn shadow_blur_is_emitted_before_a_clean_fill_event() {
    let plan = resolved(&text_fixture(false));
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let alpha_fix = graph.find("sqrt(lum").unwrap();
    let straight = graph.find("unpremultiply=inplace=1:planes=7").unwrap();
    assert!(
        alpha_fix < straight,
        "ASS alpha must be restored before {graph}"
    );
    assert!(graph.contains("unpremultiply=inplace=1:planes=7"));
    let ass = ass_script(&graph);
    let background = ass.find("Dialogue: 0").unwrap();
    let shadow = ass.find("Dialogue: 1").unwrap();
    let fill = ass.find("Dialogue: 2").unwrap();
    let shadow_events = &ass[shadow..fill];
    let fill_events = &ass[fill..];

    assert!(background < shadow && shadow < fill, "ass={ass}");
    assert!(shadow_events.contains("\\bord0\\shad0\\blur2"));
    assert!(shadow_events.contains("\\1c&H000000&"));
    assert!(fill_events.contains("\\shad0\\blur0\\bord1.5"));
    assert!(fill_events.contains("\\1c&H14F0FF&"));
    assert!(!fill_events.contains("\\blur2"));
    assert!(!ass.contains("\\xshad"));
    assert!(!ass.contains("\\yshad"));
    assert!(!ass.contains("\\4c"));
}
