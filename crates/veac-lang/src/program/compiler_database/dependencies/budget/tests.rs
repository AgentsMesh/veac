use super::*;

#[test]
fn overflow_is_not_a_valid_unlimited_usage() {
    let mut usage = Usage {
        entries: 0,
        bytes: 0,
        overflowed: false,
    };
    usage.map_entry::<(), ()>(usize::MAX);
    assert!(usage.overflowed);

    let graph = DependencyGraph::new(usize::MAX, usize::MAX);
    assert!(!graph.fits(usage));
}
