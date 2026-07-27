mod annotation;
mod clip;
pub(crate) mod extension;
mod relation;
mod track;

use veac_ir::{ProjectEnvelope, SequenceId};

use crate::{
    OtioError, OtioLossReport, OtioStack, OtioTimeline, OTIO_STACK_SCHEMA, OTIO_TIMELINE_SCHEMA,
};

#[derive(Debug, Clone, PartialEq)]
pub struct OtioExportResult {
    pub timeline: OtioTimeline,
    pub loss_report: OtioLossReport,
}

pub fn export_sequence(
    project: &ProjectEnvelope,
    sequence_id: &SequenceId,
) -> Result<OtioExportResult, OtioError> {
    veac_ir::validate(project).map_err(|error| OtioError::contract(error.to_string()))?;
    let sequence = project
        .project
        .sequences
        .iter()
        .find(|value| value.id == *sequence_id)
        .ok_or_else(|| OtioError::contract(format!("sequence {sequence_id} does not exist")))?;
    let mut losses = OtioLossReport::default();
    report_sequence_features(sequence, &mut losses);
    let annotations = annotation::select(&project.project, sequence, &mut losses);
    let relations = relation::select(&project.project, sequence, &mut losses);
    let mut ordered_tracks = sequence.tracks.iter().collect::<Vec<_>>();
    ordered_tracks.sort_by_key(|track| track.order);
    let tracks = ordered_tracks
        .iter()
        .enumerate()
        .map(|(index, track)| track::export(&project.project, track, index, &mut losses))
        .collect::<Result<Vec<_>, _>>()?;
    let mut timeline = OtioTimeline {
        schema: OTIO_TIMELINE_SCHEMA.to_owned(),
        name: sequence.name.clone(),
        tracks: OtioStack {
            schema: OTIO_STACK_SCHEMA.to_owned(),
            name: "tracks".to_owned(),
            children: tracks,
            source_range: None,
            metadata: Default::default(),
            effects: vec![],
            markers: vec![],
            enabled: true,
            extra: Default::default(),
        },
        global_start_time: None,
        metadata: Default::default(),
        extra: Default::default(),
    };
    extension::attach(
        &mut timeline,
        &project.project,
        sequence,
        relations,
        annotations,
    )?;
    Ok(OtioExportResult {
        timeline,
        loss_report: losses,
    })
}

fn report_sequence_features(sequence: &veac_ir::Sequence, losses: &mut OtioLossReport) {
    losses.push(
        "",
        "settings",
        "OTIO does not standardize VEAC canvas, frame-rate, and sample-rate settings",
        true,
    );
    if !sequence.metadata.is_empty() {
        losses.push(
            "",
            "metadata",
            "VEAC sequence metadata is preserved only in the extension",
            true,
        );
    }
}
