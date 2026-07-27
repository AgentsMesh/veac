use super::*;

fn id(value: &u32) -> &u32 {
    value
}

#[test]
fn stable_anchor_index_covers_append_before_after_and_rejections() {
    let values = [1_u32, 2_u32];
    assert_eq!(index(&values, None, None, id, "new").unwrap(), 2);
    assert_eq!(index(&values, Some(&1), None, id, "new").unwrap(), 0);
    assert_eq!(index(&values, None, Some(&1), id, "new").unwrap(), 1);
    assert!(index(&values, Some(&1), Some(&2), id, "new").is_err());
    assert!(index(&values, Some(&3), None, id, "new").is_err());
}
