use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::{ResolvedClipSource, ResolvedInputKind};

use super::support::{ass_script, bindings, emit_video_command, identity, resolved, text_fixture};

#[test]
fn text_and_caption_emit_inline_ass_with_decorations_and_common_visual_pipeline() {
    for caption in [false, true] {
        let plan = resolved(&text_fixture(caption));
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        for marker in [
            "subtitles=filename='data\\:application/x-ass;base64\\,",
            ":alpha=1:wrap_unicode=0:fontsdir=",
            "textassv",
            "cardsplit",
            "shadowv",
        ] {
            assert!(graph.contains(marker), "missing {marker}: {graph}");
        }
        assert!(!graph.contains("drawtext="), "graph={graph}");
        let ass = ass_script(&graph);
        for marker in [
            "ScriptType: v4.00+",
            "\\bord1.5",
            "\\blur2",
            "Dialogue: 1",
            "Dialogue: 2",
            "\\p1",
            "a'b:c%d\\\\e",
        ] {
            assert!(ass.contains(marker), "missing {marker}: {ass}");
        }
    }
}

#[test]
fn zero_blur_shadow_uses_clean_events_and_unframed_text_keeps_full_canvas() {
    let mut plan = resolved(&text_fixture(false));
    text_content(&mut plan)
        .style
        .shadow
        .as_mut()
        .unwrap()
        .blur_pixels = 0.0;
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let ass = ass_script(&graph);
    assert!(ass.contains("Dialogue: 1"), "ass={ass}");
    assert!(ass.contains("Dialogue: 2"), "ass={ass}");
    assert!(ass.contains("\\blur0"), "ass={ass}");
    assert!(!ass.contains("\\xshad"), "ass={ass}");
    assert!(graph.contains("s=1920x1080"), "graph={graph}");
    assert!(!graph.contains("framev"), "graph={graph}");
}

#[test]
fn bad_font_face_and_missing_font_fail_with_specific_diagnostics() {
    let mut face = resolved(&text_fixture(false));
    let font_id = text_content_ref(&face).style.font.input_id.clone();
    text_content(&mut face).style.font.face_index = u32::MAX;
    let input = face
        .inputs
        .iter_mut()
        .find(|input| input.id == font_id)
        .unwrap();
    let ResolvedInputKind::Font { face_index, .. } = &mut input.kind else {
        unreachable!()
    };
    *face_index = u32::MAX;
    let error = emit_video_command(&face, &bindings(&face)).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::InvalidResourceBinding
    );
    assert_eq!(error.diagnostics()[0].code, "TEXT_FONT_FACE_MISSING");

    let mut missing = resolved(&text_fixture(false));
    missing
        .inputs
        .retain(|input| !matches!(input.kind, ResolvedInputKind::Font { .. }));
    let error = emit_video_command(&missing, &bindings(&missing)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, "PLAN_TEXT_INVALID");
}

#[test]
fn public_text_backend_reports_font_io_failures_and_missing_glyphs() {
    let plan = resolved(&text_fixture(false));
    let font = text_content_ref(&plan).style.font.input_id.clone();
    for (path, message) in [
        (std::path::PathBuf::from("/"), "no parent directory"),
        (
            std::path::PathBuf::from("/veac/no-such-font/font.ttf"),
            "cannot verify font binding",
        ),
    ] {
        let mut local = bindings(&plan);
        let input = plan.inputs.iter().find(|input| input.id == font).unwrap();
        local.bind_original(input, path).unwrap();
        let error = emit_video_command(&plan, &local).unwrap_err();
        assert_eq!(error.diagnostics()[0].code, "TEXT_FONT_BINDING_INVALID");
        assert!(error.diagnostics()[0].message.contains(message));
    }

    let mut missing = resolved(&text_fixture(false));
    text_content(&mut missing).text = "\u{10ffff}".to_owned();
    let error = emit_video_command(&missing, &bindings(&missing)).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TEXT_GLYPH_MISSING");
    assert!(error.diagnostics()[0].message.contains("U+10FFFF"));
}

#[test]
fn font_bytes_must_match_the_resolved_identity_before_shaping() {
    let mut plan = resolved(&text_fixture(false));
    let font = text_content_ref(&plan).style.font.input_id.clone();
    plan.inputs
        .iter_mut()
        .find(|input| input.id == font)
        .unwrap()
        .observed_identity = identity('0');

    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TEXT_FONT_BINDING_INVALID");
    assert!(error.diagnostics()[0].message.contains("pinned SHA-256"));
}

#[test]
fn missing_text_visual_fails_preflight_before_font_backend_work() {
    let mut plan = resolved(&text_fixture(false));
    plan.sequences[0].tracks[1].clips[0].visual = None;
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, "PLAN_STRUCTURE_INVALID");
}

fn text_content(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedText {
    let source = &mut plan.sequences[0].tracks[1].clips[0].source;
    match source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            content
        }
        _ => unreachable!(),
    }
}

fn text_content_ref(plan: &veac_plan::ResolvedRenderPlan) -> &veac_plan::ResolvedText {
    match &plan.sequences[0].tracks[1].clips[0].source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            content
        }
        _ => unreachable!(),
    }
}
