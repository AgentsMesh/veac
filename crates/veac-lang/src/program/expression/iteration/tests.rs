use std::sync::Arc;

use super::ExpressionLoopFrame;

fn frame(loop_span: std::ops::Range<usize>, index: usize) -> ExpressionLoopFrame {
    ExpressionLoopFrame::new(
        Arc::from("definition"),
        Arc::from("main"),
        Arc::from("main.veac"),
        loop_span,
        4..11,
        index,
    )
}

#[test]
fn numeric_index_is_not_part_of_the_loop_logical_key() {
    let first = frame(0..20, 0);
    let second = frame(0..20, 7);
    let different_loop = frame(21..40, 0);

    assert_eq!(first.logical_key(), second.logical_key());
    assert_ne!(first.index(), second.index());
    assert_ne!(first.logical_key(), different_loop.logical_key());
}
