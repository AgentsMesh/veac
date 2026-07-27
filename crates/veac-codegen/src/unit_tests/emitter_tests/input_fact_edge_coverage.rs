use veac_plan::ResolvedInputKind;

use super::support::{bindings, emit_video_command, fixture, resolved, text_fixture, time};

#[test]
fn optional_font_names_are_validated_independently() {
    let mut family_plan = resolved(&text_fixture(false));
    let ResolvedInputKind::Font { family, .. } = font_kind(&mut family_plan) else {
        unreachable!()
    };
    *family = Some("  ".to_owned());
    assert_input_invalid(&family_plan);

    let mut postscript_plan = resolved(&text_fixture(false));
    let ResolvedInputKind::Font {
        family,
        postscript_name,
        ..
    } = font_kind(&mut postscript_plan)
    else {
        unreachable!()
    };
    *family = None;
    *postscript_name = Some("\t".to_owned());
    assert_input_invalid(&postscript_plan);
}

#[test]
fn explicit_zero_stream_start_times_remain_valid_input_facts() {
    let mut plan = resolved(&fixture());
    let input = &mut plan.inputs[0];
    input.video.as_mut().unwrap().start_time = Some(time(0));
    input.audio.as_mut().unwrap().start_time = Some(time(0));

    emit_video_command(&plan, &bindings(&plan)).unwrap();
}

fn font_kind(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut ResolvedInputKind {
    &mut plan
        .inputs
        .iter_mut()
        .find(|input| matches!(input.kind, ResolvedInputKind::Font { .. }))
        .unwrap()
        .kind
}

fn assert_input_invalid(plan: &veac_plan::ResolvedRenderPlan) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "PLAN_INPUT_FACTS_INVALID"));
}
