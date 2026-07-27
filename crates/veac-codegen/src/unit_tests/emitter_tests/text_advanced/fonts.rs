use veac_artifact::ExecutionBindings;
use veac_plan::PlanInputId;

use super::super::support::{
    alternate_test_font_path, bindings, emit_video_command, resolved, set_input_identity,
    text_fixture,
};
use super::text_content;

#[test]
fn unicode_boundaries_and_cross_directory_fonts_are_deterministic() {
    let mut split = resolved(&text_fixture(false));
    let content = text_content(&mut split);
    content.text = "e\u{301}".to_owned();
    content.style.spans.push(veac_plan::ResolvedTextSpan {
        start: 0,
        end: 1,
        font: None,
        font_weight: None,
        font_style: None,
        size_pixels: None,
        color: None,
    });
    assert_code(&split, &bindings(&split), "TEXT_SPAN_GRAPHEME_SPLIT");

    let mut directories = resolved(&text_fixture(false));
    let font = text_content(&mut directories).style.font.clone();
    let mut fallback = font.clone();
    fallback.input_id = PlanInputId::new("pin_other_font").unwrap();
    let content = text_content(&mut directories);
    content.style.fallback_fonts.push(fallback.clone());
    content.style.spans.push(veac_plan::ResolvedTextSpan {
        start: 0,
        end: 1,
        font: Some(fallback),
        font_weight: None,
        font_style: None,
        size_pixels: None,
        color: None,
    });
    let mut input = directories
        .inputs
        .iter()
        .find(|input| input.id == font.input_id)
        .unwrap()
        .clone();
    let other_id = PlanInputId::new("pin_other_font").unwrap();
    input.id = other_id.clone();
    directories.inputs.push(input);
    directories
        .inputs
        .sort_by(|left, right| left.id.cmp(&right.id));
    let other_path = alternate_test_font_path();
    set_input_identity(&mut directories, &other_id, &other_path);
    let mut local = bindings(&directories);
    let other = directories
        .inputs
        .iter()
        .find(|input| input.id == other_id)
        .unwrap();
    local.bind_original(other, other_path).unwrap();
    let graph = emit_video_command(&directories, &local)
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("subtitles=filename="), "graph={graph}");
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, local: &ExecutionBindings, code: &str) {
    let error = emit_video_command(plan, local).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, code);
}
