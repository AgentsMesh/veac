use super::*;

#[test]
fn fallback_and_span_fonts_from_multiple_directories_share_safe_attachments() {
    let temp = tempfile::tempdir().unwrap();
    let first_dir = temp.path().join("fallback");
    let second_dir = temp.path().join("span");
    std::fs::create_dir_all(&first_dir).unwrap();
    std::fs::create_dir_all(&second_dir).unwrap();
    let first_path = first_dir.join("first.ttf");
    let second_path = second_dir.join("second.ttf");
    std::fs::copy(small_script_font(), &first_path).unwrap();
    std::fs::copy(small_script_font(), &second_path).unwrap();

    let mut plan = resolved(&text_fixture(false));
    let fallback_id = add_font_input(&mut plan, "pin_fallback_remote");
    let span_id = add_font_input(&mut plan, "pin_span_remote");
    set_input_identity(&mut plan, &fallback_id, &first_path);
    set_input_identity(&mut plan, &span_id, &second_path);
    let primary = text_content(&mut plan).style.font.clone();
    let mut fallback = primary.clone();
    fallback.input_id = fallback_id.clone();
    let mut span_font = primary;
    span_font.input_id = span_id.clone();
    let content = text_content(&mut plan);
    content.text = "\u{108e0}\u{108e1}".to_owned();
    content.style.fallback_fonts.push(fallback);
    content.style.spans.push(veac_plan::ResolvedTextSpan {
        start: 0,
        end: 1,
        font: Some(span_font),
        font_weight: None,
        font_style: None,
        size_pixels: None,
        color: None,
    });
    let mut local = bindings(&plan);
    let fallback = plan
        .inputs
        .iter()
        .find(|input| input.id == fallback_id)
        .unwrap();
    local.bind_original(fallback, first_path).unwrap();
    let span = plan
        .inputs
        .iter()
        .find(|input| input.id == span_id)
        .unwrap();
    local.bind_original(span, second_path).unwrap();
    let graph = emit_video_command(&plan, &local)
        .unwrap()
        .filter_graph
        .unwrap();
    let ass = ass_script(&graph);
    for attachment in ["fontname: veac-font-1.ttf", "fontname: veac-font-2.ttf"] {
        assert!(ass.contains(attachment), "missing {attachment}: {ass}");
    }
}
