use std::fs;

use veac_lang::program::{apply_executable_source_edit_path, SourceTransactionError};

use super::support::{assert_unchanged, batch, Fixture};

#[test]
fn invalid_digest_path_and_font_kind_are_byte_identical_rollbacks() {
    for (function, body) in [
        (
            "voice",
            r#"{ audio_resource(identifier("voice"), resource_file("assets/new.wav"),
              sha256("bad"), stream_auto()) }"#,
        ),
        (
            "voice",
            r#"{ audio_resource(identifier("voice"), resource_file("/escape.wav"),
              sha256("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
              stream_auto()) }"#,
        ),
        (
            "caption",
            r#"{ item(identifier("caption"), item_enabled(), during(0s, 2s),
              source_caption("错误字体", style(wrong)), source_timing_native()) }"#,
        ),
    ] {
        let fixture = Fixture::new();
        let entry = fs::read(&fixture.entry).unwrap();
        let module = fs::read(&fixture.module).unwrap();
        let ir = fixture.baseline();
        let error = apply_executable_source_edit_path(
            &fixture.entry,
            &batch(&fixture.entry, function, body),
        )
        .unwrap_err();
        assert!(matches!(error, SourceTransactionError::Program(_)));
        assert_unchanged(&fixture, &entry, &module, &ir);
    }
}
