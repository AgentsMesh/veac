use veac_codegen::emitter::emit_all;
use veac_plan::canonical::SequenceId;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, fixture, resolved};

#[test]
fn unreachable_and_non_dependency_ordered_sequences_fail_closed() {
    let mut unreachable = resolved(&fixture());
    let mut child = unreachable.sequences[0].clone();
    child.id = SequenceId::new("seq_unreachable").unwrap();
    unreachable.sequences.push(child);
    assert_code(&unreachable, "PLAN_SEQUENCE_UNREACHABLE");

    let mut unordered = resolved(&fixture());
    let mut child = unordered.sequences[0].clone();
    child.id = SequenceId::new("seq_child_after_parent").unwrap();
    unordered.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    unordered.sequences.push(child);
    assert_code(&unordered, "PLAN_SEQUENCE_ORDER_INVALID");
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, expected: &str) {
    let error = emit_all(plan, &bindings(plan)).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|value| value.code == expected),
        "missing {expected}: {error}"
    );
}
