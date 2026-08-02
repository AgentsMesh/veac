use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::arguments::{CaptionCommand, CaptionFormatArg, CaptionOverlapArg};

pub(super) fn run(command: CaptionCommand) -> crate::CliResult {
    crate::commands::caption(command)
}

pub(super) fn import_command(input: &Path, output: Option<PathBuf>) -> CaptionCommand {
    CaptionCommand::Import {
        input: input.to_path_buf(),
        format: CaptionFormatArg::Srt,
        timescale: 600,
        overlap: CaptionOverlapArg::Reject,
        namespace: "caption".to_owned(),
        output,
        loss_report: None,
        allow_lossy: false,
    }
}

pub(super) fn project_and_bindings(
    temp: &TempDir,
    document: &Path,
) -> (
    PathBuf,
    PathBuf,
    veac_caption::CaptionCueId,
    veac_ir::ItemId,
) {
    let project =
        super::super::support::canonical_project(temp, super::super::support::GENERATED_SOURCE);
    let envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&project).unwrap()).unwrap();
    let captions =
        veac_caption::decode_caption_json(&std::fs::read_to_string(document).unwrap()).unwrap();
    let cue_id = captions.document.cues[0].id.clone();
    let item_id = veac_ir::ItemId::new("itm_imported_caption").unwrap();
    let bindings = veac_caption::CaptionTrackInsertionBindings {
        sequence_id: veac_ir::SequenceId::new("seq_main").unwrap(),
        track: veac_caption::CaptionTrackBindings {
            track_id: veac_ir::TrackId::new("trk_imported_captions").unwrap(),
            cue_item_ids: BTreeMap::from([(cue_id.clone(), item_id.clone())]),
            text_style: veac_ir::TextStyle::default(),
            visual: envelope.project.sequences[0].tracks[0].clips[0]
                .visual
                .clone()
                .unwrap(),
            order: 20,
        },
        before_id: None,
        after_id: None,
    };
    let path = temp.path().join("insert-bindings.json");
    std::fs::write(&path, serde_json::to_vec(&bindings).unwrap()).unwrap();
    (project, path, cue_id, item_id)
}

pub(super) fn apply_batch(project: &Path, batch: &Path) {
    let envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(project).unwrap()).unwrap();
    let batch = veac_ir::decode_edit_batch_json(&std::fs::read_to_string(batch).unwrap()).unwrap();
    let outcome = veac_ir::apply_edit_batch(&envelope, &batch);
    let next = match outcome {
        veac_ir::EditOutcome::Applied {
            project: next_project,
            ..
        } => next_project,
        _ => panic!("caption insertion must apply"),
    };
    std::fs::write(project, veac_ir::canonical_json(&next).unwrap()).unwrap();
}
