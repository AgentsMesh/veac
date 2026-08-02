use serde_json::json;
use veac_ir::StreamChoice;

use super::*;

#[test]
fn missing_and_invalid_container_formats_are_rejected() {
    let mut missing = complete_output();
    missing.as_object_mut().unwrap().remove("format");
    assert!(parse(&missing, intent(StreamChoice::Auto, StreamChoice::Auto)).is_err());

    let mut invalid = complete_output();
    invalid["format"]["format_name"] = json!("");
    assert!(parse(&invalid, intent(StreamChoice::Auto, StreamChoice::Auto)).is_err());
}
