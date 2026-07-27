use super::load::FontLimits;
use super::*;
use crate::unit_tests::emitter_tests::support::{bindings, resolved, test_font_path, text_fixture};
use veac_plan::{PlanInputId, ResolvedClipSource, ResolvedRenderPlan, ResolvedText};

#[test]
fn font_book_finds_the_first_covering_face_and_rejects_missing_faces() {
    let plan = resolved(&text_fixture(false));
    let ResolvedClipSource::Text { content } = &plan.sequences[0].tracks[1].clips[0].source else {
        unreachable!()
    };
    let typed = bindings(&plan);
    let book = FontBook::load(content, &typed).unwrap();
    assert_eq!(book.first_covering("A"), Some(0));
    assert_eq!(book.first_covering("\u{10ffff}"), None);
    assert!(!book.covers(usize::MAX, "A"));
    let mut absent = content.style.font.clone();
    absent.input_id = PlanInputId::new("pin_absent_font").unwrap();
    assert_eq!(book.index(&absent).unwrap_err().code, "TEXT_PLAN_INVALID");
}

#[test]
fn font_book_rejects_one_font_above_its_verified_read_limit() {
    let plan = resolved(&text_fixture(false));
    let typed = typed_bindings(&plan);
    let limits = FontLimits {
        file_bytes: 1,
        total_bytes: u64::MAX,
        embedded_bytes: usize::MAX,
    };
    let error = FontBook::load_with_limits(text(&plan), &typed, limits)
        .err()
        .unwrap();
    assert_eq!(error.code, "TEXT_FONT_FILE_LIMIT");
}

#[test]
fn font_book_enforces_the_aggregate_verified_read_limit() {
    let mut plan = resolved(&text_fixture(false));
    add_duplicate_font(&mut plan);
    let font_bytes = std::fs::metadata(test_font_path()).unwrap().len();
    let typed = typed_bindings(&plan);
    let limits = FontLimits {
        file_bytes: font_bytes,
        total_bytes: font_bytes * 2 - 1,
        embedded_bytes: usize::MAX,
    };
    let error = FontBook::load_with_limits(text(&plan), &typed, limits)
        .err()
        .unwrap();
    assert_eq!(error.code, "TEXT_FONT_TOTAL_LIMIT");
}

fn typed_bindings(plan: &ResolvedRenderPlan) -> veac_artifact::ExecutionBindings {
    bindings(plan)
}

fn text(plan: &ResolvedRenderPlan) -> &ResolvedText {
    let ResolvedClipSource::Text { content } = &plan.sequences[0].tracks[1].clips[0].source else {
        panic!("text fixture")
    };
    content
}

fn add_duplicate_font(plan: &mut ResolvedRenderPlan) {
    let primary = text(plan).style.font.clone();
    let id = PlanInputId::new("pin_duplicate_font").unwrap();
    let mut fallback = primary.clone();
    fallback.input_id = id.clone();
    let ResolvedClipSource::Text { content } = &mut plan.sequences[0].tracks[1].clips[0].source
    else {
        panic!("text fixture")
    };
    content.style.fallback_fonts.push(fallback);
    let mut input = plan
        .inputs
        .iter()
        .find(|value| value.id == primary.input_id)
        .unwrap()
        .clone();
    input.id = id;
    plan.inputs.push(input);
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
}
