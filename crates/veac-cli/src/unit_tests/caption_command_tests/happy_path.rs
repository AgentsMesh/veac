use std::collections::BTreeMap;

use tempfile::tempdir;

use super::support::*;
use crate::arguments::{CaptionCommand, CaptionFormatArg};

#[test]
fn caption_commands_round_trip_through_a_checked_edit_batch() {
    let temp = tempdir().unwrap();
    let sidecar = temp.path().join("captions.srt");
    let document = temp.path().join("captions.json");
    std::fs::write(&sidecar, "1\n00:00:00,000 --> 00:00:01,000\nHello VEAC\n").unwrap();
    run(import_command(&sidecar, Some(document.clone()))).unwrap();

    let (project, insertion, cue_id, item_id) = project_and_bindings(&temp, &document);
    let batch = temp.path().join("batch.json");
    run(CaptionCommand::Propose {
        project: project.clone(),
        document: document.clone(),
        bindings: insertion,
        operation_id: "op_caption_import".to_owned(),
        output: Some(batch.clone()),
    })
    .unwrap();
    apply_batch(&project, &batch);

    let captions =
        veac_caption::decode_caption_json(&std::fs::read_to_string(&document).unwrap()).unwrap();
    let extraction = veac_caption::CaptionDocumentBindings {
        cue_ids: BTreeMap::from([(item_id, cue_id)]),
        language: captions.document.language,
        overlap_policy: captions.document.overlap_policy,
        native: captions.document.native,
        styles: captions.document.styles,
    };
    let bindings = temp.path().join("extract-bindings.json");
    std::fs::write(&bindings, serde_json::to_vec(&extraction).unwrap()).unwrap();
    let extracted = temp.path().join("extracted.json");
    run(CaptionCommand::Extract {
        project,
        track: "trk_imported_captions".to_owned(),
        bindings,
        output: Some(extracted.clone()),
    })
    .unwrap();

    let output = temp.path().join("captions-out.srt");
    let losses = temp.path().join("captions-out.loss.json");
    run(CaptionCommand::Export {
        document: extracted,
        format: CaptionFormatArg::Srt,
        output: Some(output.clone()),
        loss_report: Some(losses.clone()),
        allow_lossy: true,
    })
    .unwrap();
    assert!(std::fs::read_to_string(output)
        .unwrap()
        .contains("Hello VEAC"));
    let losses: veac_caption::LossReport =
        serde_json::from_str(&std::fs::read_to_string(losses).unwrap()).unwrap();
    assert!(losses.is_empty());
}

#[test]
fn lossless_import_can_emit_to_stdout_and_an_empty_report() {
    let temp = tempdir().unwrap();
    let sidecar = temp.path().join("caption.srt");
    let report = temp.path().join("loss.json");
    std::fs::write(&sidecar, "1\n00:00:00,000 --> 00:00:01,000\nHello\n").unwrap();
    let mut command = import_command(&sidecar, None);
    let CaptionCommand::Import {
        loss_report,
        allow_lossy,
        ..
    } = &mut command
    else {
        panic!()
    };
    *loss_report = Some(report.clone());
    *allow_lossy = true;
    run(command).unwrap();
    let report: veac_caption::LossReport =
        serde_json::from_str(&std::fs::read_to_string(report).unwrap()).unwrap();
    assert!(report.is_empty());
}
