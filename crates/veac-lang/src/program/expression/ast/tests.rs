use super::Expression;

#[test]
fn expression_nodes_keep_recursive_parser_frames_bounded() {
    let size = std::mem::size_of::<Expression>();
    assert!(size <= 104, "Expression grew to {size} bytes");
}
