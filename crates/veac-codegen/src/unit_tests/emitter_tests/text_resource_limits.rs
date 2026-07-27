use veac_plan::{PlanInputId, ResolvedClipSource};

use super::support::{
    alternate_test_font_path, ass_script, bindings, emit_video_command, resolved,
    set_input_identity, text_fixture,
};

#[test]
fn small_cross_directory_font_is_embedded_in_the_inline_ass() {
    let temp = tempfile::tempdir().unwrap();
    let custom = temp.path().join("custom.ttf");
    std::fs::copy(small_script_font(), &custom).unwrap();
    let mut plan = resolved(&text_fixture(false));
    text_content(&mut plan).text = "\u{108e0}".to_owned();
    let id = add_fallback_input(&mut plan, "pin_custom_font");
    set_input_identity(&mut plan, &id, &custom);
    let mut local = bindings(&plan);
    let input = plan.inputs.iter().find(|input| input.id == id).unwrap();
    local.bind_original(input, custom).unwrap();
    let graph = emit_video_command(&plan, &local)
        .unwrap()
        .filter_graph
        .unwrap();
    let ass = ass_script(&graph);
    assert!(ass.contains("[Fonts]\nfontname: veac-font-1.ttf"));
}

#[test]
fn large_cross_directory_font_fails_before_inline_encoding() {
    let temp = tempfile::tempdir().unwrap();
    let custom = temp.path().join("large.ttf");
    std::fs::copy(alternate_test_font_path(), &custom).unwrap();
    let mut plan = resolved(&text_fixture(false));
    let id = add_fallback_input(&mut plan, "pin_large_font");
    set_input_identity(&mut plan, &id, &custom);
    let mut local = bindings(&plan);
    let input = plan.inputs.iter().find(|input| input.id == id).unwrap();
    local.bind_original(input, custom).unwrap();
    let error = emit_video_command(&plan, &local).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TEXT_FONT_EMBED_LIMIT");
}

fn text_content(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedText {
    match &mut plan.sequences[0].tracks[1].clips[0].source {
        ResolvedClipSource::Text { content } => content,
        _ => panic!("text fixture"),
    }
}

fn add_fallback_input(plan: &mut veac_plan::ResolvedRenderPlan, id: &str) -> PlanInputId {
    let primary = text_content(plan).style.font.clone();
    let id = PlanInputId::new(id).unwrap();
    let mut fallback = primary.clone();
    fallback.input_id = id.clone();
    text_content(plan).style.fallback_fonts.push(fallback);
    let mut input = plan
        .inputs
        .iter()
        .find(|input| input.id == primary.input_id)
        .unwrap()
        .clone();
    input.id = id.clone();
    plan.inputs.push(input);
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    id
}

fn small_script_font() -> std::path::PathBuf {
    [
        "/System/Library/Fonts/Supplemental/NotoSansHatran-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansHatran-Regular.ttf",
    ]
    .iter()
    .map(std::path::PathBuf::from)
    .find(|path| path.is_file())
    .expect("CI must install a small Noto script font")
}
