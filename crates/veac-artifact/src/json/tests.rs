use std::io::Write;

use super::BoundedVec;

#[test]
fn bounded_json_writer_flush_is_a_noop() {
    let mut output = BoundedVec::new(8);
    output.write_all(b"json").unwrap();
    output.flush().unwrap();
    assert_eq!(output.bytes, b"json");
}
