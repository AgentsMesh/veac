mod bound;
pub(crate) mod defaults;
mod extension;
mod report;

use veac_ir::{Annotation, Material, MulticamGroup, Relation, Sequence};

use crate::{OtioError, OtioImportBindings, OtioLossReport, OtioTimeline};

#[derive(Debug, Clone, PartialEq)]
pub struct OtioImportResult {
    pub sequence: Sequence,
    pub relations: Vec<Relation>,
    pub materials: Vec<Material>,
    pub multicam_groups: Vec<MulticamGroup>,
    pub annotations: Vec<Annotation>,
    pub loss_report: OtioLossReport,
}

pub fn import_veac_extension(timeline: &OtioTimeline) -> Result<OtioImportResult, OtioError> {
    validate_header(timeline)?;
    extension::import(timeline)
}

pub fn import_bound_timeline(
    timeline: &OtioTimeline,
    bindings: &OtioImportBindings,
) -> Result<OtioImportResult, OtioError> {
    validate_header(timeline)?;
    bindings.validate()?;
    bound::import(timeline, bindings)
}

pub(crate) fn validate_header(value: &OtioTimeline) -> Result<(), OtioError> {
    if value.schema != crate::OTIO_TIMELINE_SCHEMA
        || value.tracks.schema != crate::OTIO_STACK_SCHEMA
        || value
            .tracks
            .children
            .iter()
            .any(|track| track.schema != crate::OTIO_TRACK_SCHEMA)
    {
        return Err(OtioError::contract(
            "only Timeline.1 with Stack.1 and Track.1 is supported",
        ));
    }
    if let Some(time) = &value.global_start_time {
        crate::time::validate_time(time)?;
    }
    if let Some(range) = &value.tracks.source_range {
        crate::time::validate_range(range)?;
    }
    for track in &value.tracks.children {
        validate_track(track)?;
    }
    Ok(())
}

fn validate_track(track: &crate::OtioTrack) -> Result<(), OtioError> {
    if let Some(range) = &track.source_range {
        crate::time::validate_range(range)?;
    }
    for item in &track.children {
        match item {
            crate::OtioItem::Clip {
                source_range,
                media_reference,
                ..
            } => {
                crate::time::validate_range(source_range)?;
                validate_reference(media_reference)?;
            }
            crate::OtioItem::Gap { source_range, .. } => {
                crate::time::validate_range(source_range)?;
            }
            crate::OtioItem::Transition {
                in_offset,
                out_offset,
                ..
            } => {
                crate::time::validate_time(in_offset)?;
                crate::time::validate_time(out_offset)?;
            }
            crate::OtioItem::Unknown => {
                return Err(OtioError::contract("unknown OTIO item schema"));
            }
        }
    }
    Ok(())
}

fn validate_reference(reference: &crate::OtioMediaReference) -> Result<(), OtioError> {
    match reference {
        crate::OtioMediaReference::External {
            target_url,
            available_range,
            ..
        } => {
            if target_url.trim().is_empty() {
                return Err(OtioError::contract(
                    "external media target URL cannot be empty",
                ));
            }
            if let Some(range) = available_range {
                crate::time::validate_range(range)?;
            }
            Ok(())
        }
        crate::OtioMediaReference::Missing { .. } => Ok(()),
        crate::OtioMediaReference::Unknown => {
            Err(OtioError::contract("unknown OTIO media-reference schema"))
        }
    }
}
