use super::{sheet_dimension, sheet_length};

#[test]
fn contact_sheet_arithmetic_is_bounded_before_allocation() {
    assert_eq!(sheet_dimension(10, 4, "width").unwrap(), 40);
    assert!(sheet_dimension(u32::MAX, 2, "width").is_err());
    assert_eq!(sheet_length(4, 4).unwrap(), 64);
    assert!(sheet_length(10_000, 10_000).is_err());
    assert!(sheet_length(u32::MAX, u32::MAX).is_err());
}
