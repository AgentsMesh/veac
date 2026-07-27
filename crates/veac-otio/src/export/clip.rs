use std::collections::BTreeMap;

use veac_ir::{
    Clip, ClipSource, MaterialSource, PlaybackDirection, Project, Rational, SourceTimeMap,
};

use crate::{OtioError, OtioItem, OtioLossReport, OtioMediaReference};

pub(super) fn export(
    project: &Project,
    clip: &Clip,
    pointer: &str,
    losses: &mut OtioLossReport,
) -> Result<OtioItem, OtioError> {
    let (media_reference, source_start) = reference(project, clip, pointer, losses)?;
    report_clip_features(clip, pointer, losses);
    let source_range = veac_ir::TimeRange::new(source_start, clip.record_range.duration)
        .map_err(OtioError::time)?;
    Ok(OtioItem::Clip {
        name: clip.id.to_string(),
        source_range: crate::time::export_range(source_range)?,
        media_reference,
        metadata: BTreeMap::from([(
            "veac.item_id".to_owned(),
            serde_json::Value::String(clip.id.to_string()),
        )]),
        effects: vec![],
        markers: vec![],
        enabled: clip.enabled,
        extra: BTreeMap::new(),
    })
}

fn reference(
    project: &Project,
    clip: &Clip,
    pointer: &str,
    losses: &mut OtioLossReport,
) -> Result<(OtioMediaReference, veac_ir::RationalTime), OtioError> {
    let zero = veac_ir::RationalTime::zero(project.timebase).map_err(OtioError::time)?;
    let ClipSource::Media { material_id } = &clip.source else {
        losses.push(
            pointer,
            "source",
            "OTIO external references cannot represent this VEAC source kind",
            true,
        );
        return Ok((missing(), zero));
    };
    let material = project
        .materials
        .iter()
        .find(|value| value.id == *material_id)
        .ok_or_else(|| OtioError::contract(format!("material {material_id} is missing")))?;
    let target_url = match &material.source {
        MaterialSource::File { uri } | MaterialSource::Remote { uri } => uri.clone(),
    };
    let start = linear_start(clip, pointer, losses)?.unwrap_or(zero);
    Ok((external(target_url, material_id.to_string()), start))
}

fn linear_start(
    clip: &Clip,
    pointer: &str,
    losses: &mut OtioLossReport,
) -> Result<Option<veac_ir::RationalTime>, OtioError> {
    let mapping = clip
        .source_mapping
        .as_ref()
        .expect("canonical media clips have source mappings");
    let SourceTimeMap::Linear {
        source_start,
        rate,
        repeat,
        direction,
    } = &mapping.time_map
    else {
        losses.push(
            pointer,
            "source_mapping",
            "OTIO has no source-time curve",
            true,
        );
        return Ok(None);
    };
    if *rate != Rational::new(1, 1).map_err(OtioError::time)?
        || *repeat != 1
        || *direction != PlaybackDirection::Forward
    {
        losses.push(
            pointer,
            "source_mapping",
            "OTIO source ranges do not preserve VEAC speed, repeat, or direction",
            true,
        );
    }
    Ok(Some(*source_start))
}

fn report_clip_features(clip: &Clip, pointer: &str, losses: &mut OtioLossReport) {
    for (present, field, reason) in [
        (
            clip.visual.is_some(),
            "visual",
            "OTIO Clip has no VEAC visual pipeline",
        ),
        (
            clip.audio.is_some(),
            "audio",
            "OTIO Clip has no VEAC audio pipeline",
        ),
        (
            !clip.effects.is_empty(),
            "effects",
            "OTIO effects are not VEAC typed effects",
        ),
        (
            !clip.metadata.is_empty(),
            "metadata",
            "VEAC clip metadata is preserved only in the extension",
        ),
        (
            clip.source_mapping.as_ref().is_some_and(|mapping| {
                mapping.frame_synthesis != veac_ir::FrameSynthesisPolicy::Nearest
            }),
            "source_mapping.frame_synthesis",
            "OTIO has no VEAC frame-synthesis policy",
        ),
    ] {
        if present {
            losses.push(pointer, field, reason, true);
        }
    }
}

fn external(target_url: String, material_id: String) -> OtioMediaReference {
    OtioMediaReference::External {
        target_url,
        available_range: None,
        metadata: BTreeMap::from([(
            "veac.material_id".to_owned(),
            serde_json::Value::String(material_id),
        )]),
        extra: BTreeMap::new(),
    }
}

fn missing() -> OtioMediaReference {
    OtioMediaReference::Missing {
        metadata: BTreeMap::new(),
        extra: BTreeMap::new(),
    }
}
