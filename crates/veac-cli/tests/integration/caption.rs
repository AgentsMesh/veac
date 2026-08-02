use std::collections::BTreeMap;

use super::support::*;

#[test]
fn caption_sidecar_round_trip_uses_a_checked_edit_batch() {
    let temp = tempdir().unwrap();
    let sidecar = temp.path().join("captions.srt");
    let document = temp.path().join("captions.json");
    let insertion = temp.path().join("insert-bindings.json");
    let batch = temp.path().join("caption-edit.json");
    let extracted = temp.path().join("extracted.json");
    let extraction = temp.path().join("extract-bindings.json");
    let exported = temp.path().join("exported.srt");
    let export_losses = temp.path().join("export-losses.json");
    std::fs::write(&sidecar, "1\n00:00:00,000 --> 00:00:01,000\nHello VEAC\n").unwrap();
    veac()
        .args([
            "caption",
            "import",
            sidecar.to_str().unwrap(),
            "--format",
            "srt",
            "--timescale",
            "600",
            "--output",
            document.to_str().unwrap(),
        ])
        .assert()
        .success();
    let captions =
        veac_caption::decode_caption_json(&std::fs::read_to_string(&document).unwrap()).unwrap();

    let project = compile_ir(&temp, GENERATED_SOURCE);
    let envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&project).unwrap()).unwrap();
    let visual = envelope.project.sequences[0].tracks[0].clips[0]
        .visual
        .clone()
        .unwrap();
    let cue_id = captions.document.cues[0].id.clone();
    let item_id = veac_ir::ItemId::new("itm_imported_caption").unwrap();
    let bindings = veac_caption::CaptionTrackInsertionBindings {
        sequence_id: veac_ir::SequenceId::new("seq_main").unwrap(),
        track: veac_caption::CaptionTrackBindings {
            track_id: veac_ir::TrackId::new("trk_imported_captions").unwrap(),
            cue_item_ids: BTreeMap::from([(cue_id.clone(), item_id.clone())]),
            text_style: veac_ir::TextStyle::default(),
            visual,
            order: 20,
        },
        before_id: None,
        after_id: None,
    };
    std::fs::write(&insertion, serde_json::to_vec(&bindings).unwrap()).unwrap();
    veac()
        .args([
            "caption",
            "propose",
            project.to_str().unwrap(),
            document.to_str().unwrap(),
            insertion.to_str().unwrap(),
            "--operation-id",
            "op_import_captions",
            "--output",
            batch.to_str().unwrap(),
        ])
        .assert()
        .success();
    veac()
        .args(["edit", project.to_str().unwrap(), batch.to_str().unwrap()])
        .assert()
        .success();

    let extraction_bindings = veac_caption::CaptionDocumentBindings {
        cue_ids: BTreeMap::from([(item_id, cue_id)]),
        language: captions.document.language.clone(),
        overlap_policy: captions.document.overlap_policy,
        settings: captions.document.settings.clone(),
        styles: captions.document.styles.clone(),
    };
    std::fs::write(
        &extraction,
        serde_json::to_vec(&extraction_bindings).unwrap(),
    )
    .unwrap();
    veac()
        .args([
            "caption",
            "extract",
            project.to_str().unwrap(),
            "--track",
            "trk_imported_captions",
            extraction.to_str().unwrap(),
            "--output",
            extracted.to_str().unwrap(),
        ])
        .assert()
        .success();
    veac()
        .args([
            "caption",
            "export",
            extracted.to_str().unwrap(),
            "--format",
            "srt",
            "--output",
            exported.to_str().unwrap(),
            "--allow-lossy",
            "--loss-report",
            export_losses.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(std::fs::read_to_string(exported)
        .unwrap()
        .contains("Hello VEAC"));
    let losses: veac_caption::LossReport =
        serde_json::from_str(&std::fs::read_to_string(export_losses).unwrap()).unwrap();
    assert_eq!(losses.losses[0].field, "settings.srt.index");
}

#[test]
fn caption_export_requires_an_explicit_loss_report_for_lossy_formats() {
    let temp = tempdir().unwrap();
    let sidecar = temp.path().join("captions.vtt");
    let document = temp.path().join("captions.json");
    let output = temp.path().join("captions.srt");
    let losses = temp.path().join("losses.json");
    std::fs::write(
        &sidecar,
        "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n<v Speaker>Hello\n",
    )
    .unwrap();
    veac()
        .args([
            "caption",
            "import",
            sidecar.to_str().unwrap(),
            "--format",
            "web-vtt",
            "--output",
            document.to_str().unwrap(),
        ])
        .assert()
        .success();
    veac()
        .args([
            "caption",
            "export",
            document.to_str().unwrap(),
            "--format",
            "srt",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("CAPTION_LOSS_UNACKNOWLEDGED"));
    assert!(!output.exists());
    veac()
        .args([
            "caption",
            "export",
            document.to_str().unwrap(),
            "--format",
            "srt",
            "--output",
            output.to_str().unwrap(),
            "--allow-lossy",
            "--loss-report",
            losses.to_str().unwrap(),
        ])
        .assert()
        .success();
    let report: veac_caption::LossReport =
        serde_json::from_str(&std::fs::read_to_string(losses).unwrap()).unwrap();
    assert!(!report.is_empty());
}
