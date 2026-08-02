#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::FillMode;
use veac_template::{
    decode_template_fill_request_json, template_fill_request_json_schema, TemplateErrorKind,
};

#[test]
fn request_round_trips_strictly_and_has_a_schema() {
    let project = project(FillMode::FitDuration, false);
    let value = request(&project, video(six_seconds(), 1920, 1080));
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(decode_template_fill_request_json(&json).unwrap(), value);
    let schema = template_fill_request_json_schema().unwrap();
    assert_eq!(schema["title"], "TemplateFillRequest");
    assert!(schema.to_string().contains("media_bindings"));
    let mut invalid = value.clone();
    invalid.schema = "wrong".to_owned();
    let error = veac_template::propose_template_fill(&project, &invalid).unwrap_err();
    assert!(format!("{error}").contains("template fill request"));
}

#[test]
fn decoder_rejects_unknown_duplicate_and_unsupported_contracts() {
    let project = project(FillMode::FitDuration, false);
    let value = request(&project, video(six_seconds(), 1920, 1080));
    let mut object = serde_json::to_value(&value).unwrap();
    object["unknown"] = serde_json::json!(true);
    assert_eq!(
        decode_template_fill_request_json(&object.to_string())
            .unwrap_err()
            .kind,
        TemplateErrorKind::InvalidRequest
    );
    let duplicate =
        serde_json::to_string(&value)
            .unwrap()
            .replacen("{", "{\"schema\":\"duplicate\",", 1);
    assert_eq!(
        decode_template_fill_request_json(&duplicate)
            .unwrap_err()
            .kind,
        TemplateErrorKind::InvalidRequest
    );
    let mut object = serde_json::to_value(&value).unwrap();
    object["schema"] = serde_json::json!("wrong");
    assert_eq!(
        decode_template_fill_request_json(&object.to_string())
            .unwrap_err()
            .kind,
        TemplateErrorKind::InvalidRequest
    );
    let mut object = serde_json::to_value(&value).unwrap();
    object["operation_id"] = serde_json::json!("invalid");
    assert_eq!(
        decode_template_fill_request_json(&object.to_string())
            .unwrap_err()
            .kind,
        TemplateErrorKind::InvalidRequest
    );
}
