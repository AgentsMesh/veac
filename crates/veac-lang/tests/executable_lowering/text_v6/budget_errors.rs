use super::{assert_text_error, SOURCE};

#[test]
fn decoration_and_run_style_budgets_fail_before_publication() {
    for (from, to) in [
        (
            "text_background_present(#102030cc, 12px)",
            "text_background_present(#102030cc, 4097px)",
        ),
        (
            "text_outline_present(#48d7caff, 2px)",
            "text_outline_present(#48d7caff, 257px)",
        ),
        (
            "shadow_present(8px, 55%, vector(4.0, 6.0), #000000cc)",
            "shadow_present(257px, 55%, vector(4.0, 6.0), #000000cc)",
        ),
        (
            "font_style_italic(), 48px, #ffe066ff",
            "font_style_italic(), 0px, #ffe066ff",
        ),
        (
            "font_style_italic(), 48px, #ffe066ff",
            "font_style_italic(), 4097px, #ffe066ff",
        ),
    ] {
        assert_text_error(&changed(from, to));
    }
}

#[test]
fn metrics_and_box_upper_bounds_fail_closed() {
    for (from, to) in [
        ("42px, 1px, 1.2", "4097px, 1px, 1.2"),
        ("42px, 1px, 1.2", "42px, 4097px, 1.2"),
        ("42px, 1px, 1.2", "42px, 1px, 17.0"),
        (
            "text_box_fixed(520px, 120px)",
            "text_box_fixed(8192px, 8192px)",
        ),
        (
            "text_box_fixed(520px, 120px)",
            "text_box_fixed(8193px, 120px)",
        ),
    ] {
        assert_text_error(&changed(from, to));
    }
}

#[test]
fn text_content_speaker_and_fallback_budgets_are_enforced() {
    assert_text_error(&changed("\"逐词动画：代码化视频\"", "\"\""));
    for speaker in ["   ".to_owned(), "x".repeat(257), "\\n".to_owned()] {
        assert_text_error(&changed(
            "source_caption_speaker(content, \"VEAC\"",
            &format!("source_caption_speaker(content, \"{speaker}\""),
        ));
    }
    let fallbacks = (0..33)
        .map(|index| format!("font_family(\"fallback-{index}\")"))
        .collect::<Vec<_>>()
        .join(", ");
    assert_text_error(&changed(
        "[font_family(\"PingFang SC\"), font_family(\"Noto Sans CJK SC\")]",
        &format!("[{fallbacks}]"),
    ));
}

fn changed(from: &str, to: &str) -> String {
    assert!(SOURCE.contains(from), "missing fixture fragment: {from}");
    SOURCE.replacen(from, to, 1)
}
