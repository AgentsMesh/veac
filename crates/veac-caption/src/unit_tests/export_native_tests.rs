use crate::{test_support::*, *};

#[test]
fn preserves_srt_native_index() {
    let mut value = envelope();
    value.document.cues[0].native = Some(CaptionNativeCue::Srt { index: 42 });
    let output = export_caption(&value, CaptionFormat::Srt).unwrap();
    assert!(output.content.starts_with("42\n00:00:00,000"));
    let imported = import_caption(
        &output.content,
        CaptionFormat::Srt,
        &ImportOptions::default(),
    )
    .unwrap();
    assert!(matches!(
        imported.document.cues[0].native,
        Some(CaptionNativeCue::Srt { index: 42 })
    ));
}

#[test]
fn roundtrips_webvtt_header_identifier_and_settings() {
    let mut value = envelope();
    value.document.native = Some(CaptionDocumentNative::WebVtt {
        header: WebVttHeader {
            description: Some("VEAC".to_owned()),
        },
    });
    value.document.cues[0].native = Some(CaptionNativeCue::WebVtt {
        identifier: Some(CaptionNativeId("cue-alpha".to_owned())),
        settings: Some(WebVttCueSettings {
            line: Some("80%".to_owned()),
            position: Some("20%".to_owned()),
            size: Some("60%".to_owned()),
            align: Some(WebVttTextAlign::Center),
            vertical: Some(WebVttVertical::Rl),
            region: Some(WebVttRegionId("main".to_owned())),
        }),
    });
    let output = export_caption(&value, CaptionFormat::WebVtt).unwrap();
    assert!(output.content.starts_with("WEBVTT VEAC\n\ncue-alpha\n"));
    let imported = import_caption(
        &output.content,
        CaptionFormat::WebVtt,
        &ImportOptions::default(),
    )
    .unwrap();
    assert_eq!(imported.document.native, value.document.native);
    assert_eq!(
        imported.document.cues[0].native,
        value.document.cues[0].native
    );
}

#[test]
fn roundtrips_ass_script_info_and_event_fields() {
    let mut value = envelope();
    value.document.native = Some(CaptionDocumentNative::Ass {
        info: AssScriptInfo {
            title: Some("VEAC".to_owned()),
            script_type: Some(AssScriptType::V4Plus),
            wrap_style: Some(2),
            scaled_border_and_shadow: Some(true),
            play_res_x: Some(1920),
            play_res_y: Some(1080),
            ycbcr_matrix: Some(AssYcbcrMatrix::Tv709),
        },
    });
    value.document.cues[0].native = Some(CaptionNativeCue::Ass {
        settings: AssCueSettings {
            comment: true,
            layer: Some(3),
            margin_left: Some(10),
            margin_right: Some(20),
            margin_vertical: Some(30),
            effect: Some("Scroll".to_owned()),
        },
    });
    let output = export_caption(&value, CaptionFormat::Ass).unwrap();
    assert!(output.content.contains("YCbCr Matrix: TV.709"));
    assert!(output
        .content
        .contains("Comment: 3,0:00:00.00,0:00:01.00,Default,,10,20,30,Scroll,"));
    let imported = import_caption(
        &output.content,
        CaptionFormat::Ass,
        &ImportOptions::default(),
    )
    .unwrap();
    assert_eq!(imported.document.native, value.document.native);
    assert_eq!(
        imported.document.cues[0].native,
        value.document.cues[0].native
    );
}

#[test]
fn reports_every_ass_native_field_when_exporting_srt() {
    let mut value = envelope();
    value.document.native = Some(CaptionDocumentNative::Ass {
        info: AssScriptInfo {
            title: Some("VEAC".to_owned()),
            script_type: Some(AssScriptType::V4Plus),
            wrap_style: Some(2),
            scaled_border_and_shadow: Some(false),
            play_res_x: Some(1920),
            play_res_y: Some(1080),
            ycbcr_matrix: Some(AssYcbcrMatrix::Pc601),
        },
    });
    value.document.cues[0].native = Some(CaptionNativeCue::Ass {
        settings: AssCueSettings {
            comment: true,
            layer: Some(3),
            margin_left: Some(10),
            margin_right: Some(20),
            margin_vertical: Some(30),
            effect: Some("Scroll".to_owned()),
        },
    });
    let report = export_caption(&value, CaptionFormat::Srt)
        .unwrap()
        .loss_report;
    let fields: Vec<_> = report
        .losses
        .iter()
        .map(|loss| loss.field.as_str())
        .collect();
    for expected in [
        "native.ass.info.title",
        "native.ass.info.script_type",
        "native.ass.info.wrap_style",
        "native.ass.info.scaled_border_and_shadow",
        "native.ass.info.play_res_x",
        "native.ass.info.play_res_y",
        "native.ass.info.ycbcr_matrix",
        "native.ass.comment",
        "native.ass.layer",
        "native.ass.margin_left",
        "native.ass.margin_right",
        "native.ass.margin_vertical",
        "native.ass.effect",
    ] {
        assert!(fields.contains(&expected), "missing {expected}: {fields:?}");
    }
}
