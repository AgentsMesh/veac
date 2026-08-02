use std::path::{Path, PathBuf};

use veac_ir::{
    EditBatch, EditOperation, EditOutcome, OperationId, RationalTime, StructureEdit, TimeRange,
    TrackId,
};

use crate::error::{CliError, CliResult};

pub(super) fn propose(
    project: &Path,
    document: &Path,
    bindings: &Path,
    operation_id: &str,
    output: Option<&Path>,
) -> CliResult {
    let loaded = crate::canonical::load_local(project)?;
    let document_file = crate::fs::canonical_file(document, "caption document")?;
    let bindings_file = crate::fs::canonical_file(bindings, "caption track bindings")?;
    let document = crate::fs::read_utf8(&document_file, "caption document")?;
    let mut document =
        veac_caption::decode_caption_json(&document).map_err(super::io::caption_error)?;
    rescale_caption(&mut document, loaded.envelope.project.timebase)?;
    let bindings: veac_caption::CaptionTrackInsertionBindings =
        super::io::read_json(&bindings_file, "caption track bindings")?;
    let track = veac_caption::to_caption_track(&document, &bindings.track)
        .map_err(super::io::caption_error)?;
    let batch = EditBatch {
        operation_id: OperationId::new(operation_id).map_err(super::interchange::invalid_id)?,
        base_revision: loaded.envelope.project.revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::EditStructure {
            edit: StructureEdit::InsertTrack {
                sequence_id: bindings.sequence_id,
                track: Box::new(track),
                before_id: bindings.before_id,
                after_id: bindings.after_id,
            },
        }],
    };
    ensure_applicable(&loaded.envelope, &batch)?;
    let json = match veac_ir::canonical_edit_batch_json(&batch) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("CAPTION_EDIT_BATCH", error.to_string())),
    };
    let protected = protected_paths(
        &loaded.envelope,
        &loaded.project_file,
        &[document_file, bindings_file],
    );
    super::io::write_outputs(&json, output, None, &protected)
}

fn rescale_caption(value: &mut veac_caption::CaptionEnvelope, timescale: u32) -> CliResult {
    if value.document.timescale == timescale {
        return Ok(());
    }
    for cue in &mut value.document.cues {
        cue.range = rescale_range(cue.range, timescale)?;
        for word in &mut cue.words {
            word.range = rescale_range(word.range, timescale)?;
        }
    }
    value.document.timescale = timescale;
    Ok(())
}

fn rescale_range(value: TimeRange, timescale: u32) -> CliResult<TimeRange> {
    Ok(TimeRange {
        start: rescale_time(value.start, timescale)?,
        duration: rescale_time(value.duration, timescale)?,
    })
}

fn rescale_time(value: RationalTime, timescale: u32) -> CliResult<RationalTime> {
    let numerator = i128::from(value.value) * i128::from(timescale);
    let denominator = i128::from(value.timescale);
    if numerator % denominator != 0 {
        return Err(CliError::new(
            "CAPTION_TIMEBASE",
            format!(
                "caption time {}/{} is not exact at timescale {timescale}",
                value.value, value.timescale
            ),
        ));
    }
    let scaled = i64::try_from(numerator / denominator)
        .map_err(|_| CliError::new("CAPTION_TIMEBASE", "caption time exceeds range"))?;
    RationalTime::new(scaled, timescale)
        .map_err(|error| CliError::new("CAPTION_TIMEBASE", error.to_string()))
}

pub(super) fn extract(
    project: &Path,
    track: &str,
    bindings: &Path,
    output: Option<&Path>,
) -> CliResult {
    let loaded = crate::canonical::load_local(project)?;
    let track_id = TrackId::new(track).map_err(super::interchange::invalid_id)?;
    let track = loaded
        .envelope
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find(|value| value.id == track_id)
        .ok_or_else(|| {
            CliError::new(
                "CAPTION_TRACK_MISSING",
                format!("track {track_id} is missing"),
            )
        })?;
    let bindings_file = crate::fs::canonical_file(bindings, "caption document bindings")?;
    let bindings: veac_caption::CaptionDocumentBindings =
        super::io::read_json(&bindings_file, "caption document bindings")?;
    let document =
        veac_caption::from_caption_track(track, &bindings).map_err(super::io::caption_error)?;
    let json = veac_caption::canonical_caption_json(&document).map_err(super::io::caption_error)?;
    let protected = protected_paths(&loaded.envelope, &loaded.project_file, &[bindings_file]);
    super::io::write_outputs(&json, output, None, &protected)
}

fn ensure_applicable(project: &veac_ir::ProjectEnvelope, batch: &EditBatch) -> CliResult {
    match veac_ir::apply_edit_batch(project, batch) {
        EditOutcome::Applied { .. } | EditOutcome::NoChange { .. } => Ok(()),
        EditOutcome::Conflict { diagnostics, .. } | EditOutcome::Rejected { diagnostics, .. } => {
            Err(crate::diagnostic::ir(&diagnostics))
        }
    }
}

fn protected_paths(
    project: &veac_ir::ProjectEnvelope,
    project_file: &Path,
    extra: &[PathBuf],
) -> Vec<PathBuf> {
    let mut paths = vec![project_file.to_path_buf()];
    paths.extend(extra.iter().cloned());
    paths.extend(crate::canonical::local_material_paths(
        project,
        project_file,
    ));
    paths
}
