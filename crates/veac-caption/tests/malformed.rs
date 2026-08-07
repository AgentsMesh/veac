use veac_caption::*;

#[test]
fn rejects_malformed_inputs_for_every_public_format() {
    let cases = [
        (CaptionFormat::Srt, "1\nnot a timestamp\ntext\n"),
        (
            CaptionFormat::WebVtt,
            "NOT-VTT\n\n00:00:00.000 --> 00:00:01.000\nx\n",
        ),
        (CaptionFormat::Ass, "[Events]\nDialogue: malformed\n"),
    ];
    for (format, input) in cases {
        let result = import_caption(input, format, &ImportOptions::default());
        assert!(
            matches!(result, Err(CaptionError::Parse { .. })),
            "{format:?}: {result:?}"
        );
    }
}

#[test]
fn rejects_unsorted_cues_instead_of_reordering_silently() {
    let input =
        "1\n00:00:02,000 --> 00:00:03,000\nlater\n\n2\n00:00:00,000 --> 00:00:01,000\nearlier\n";
    let error = import_caption(input, CaptionFormat::Srt, &ImportOptions::default()).unwrap_err();
    let CaptionError::Validation(errors) = error else {
        panic!("expected validation error");
    };
    assert!(errors.0.iter().any(|issue| issue.code == "ORDER"));
}

#[test]
fn schema_rejects_unknown_fields() {
    let json = r#"{
      "schema":"https://veac.dev/schemas/caption-document",
      "schema_version":2,
      "document":{
        "timescale":1000,
        "language":null,
        "overlap_policy":"reject",
        "native":null,
        "styles":[],
        "cues":[],
        "unknown":true
      }
    }"#;
    assert!(matches!(
        decode_caption_json(json),
        Err(CaptionError::Json(_))
    ));

    let mut legacy = serde_json::to_value(CaptionEnvelope::new(CaptionDocument::new(
        1000,
        OverlapPolicy::Reject,
    )))
    .unwrap();
    legacy["document"]
        .as_object_mut()
        .unwrap()
        .insert("settings".to_owned(), serde_json::json!({}));
    assert!(matches!(
        decode_caption_json(&legacy.to_string()),
        Err(CaptionError::Json(_))
    ));
}
