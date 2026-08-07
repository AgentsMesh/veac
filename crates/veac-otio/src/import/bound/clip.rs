use std::collections::BTreeMap;

use veac_ir::*;

use crate::{OtioError, OtioItem, OtioMediaReference};

pub(super) fn import(
    item: &OtioItem,
    track_kind: TrackKind,
    order: i32,
    record_start: RationalTime,
    timebase: u32,
    pointer: &str,
    context: &mut super::Context<'_>,
) -> Result<Clip, OtioError> {
    let OtioItem::Clip {
        source_range,
        media_reference,
        metadata: _,
        effects,
        markers,
        enabled,
        extra,
        ..
    } = item
    else {
        return Err(OtioError::contract("clip importer received another item"));
    };
    let range = crate::time::import_range(source_range.clone(), timebase)?;
    if range.start.value < 0 {
        return Err(OtioError::time("OTIO source time cannot be negative"));
    }
    let id = context
        .bindings
        .clip_ids
        .get(pointer)
        .cloned()
        .ok_or_else(|| OtioError::contract(format!("missing clip binding for {pointer}")))?;
    context.used_clips.insert(pointer.to_owned());
    let (material, target_url) = resolve_media(media_reference, &context.media, pointer)?;
    context.used_media.insert(target_url);
    compatible(material.kind, track_kind, pointer)?;
    super::super::report::extras(&mut context.losses, pointer, extra);
    super::super::report::values(&mut context.losses, pointer, "effects", effects);
    super::super::report::values(&mut context.losses, pointer, "markers", markers);
    super::report::clip(item, pointer, &mut context.losses);
    context.losses.push(
        pointer,
        if track_kind == TrackKind::Video {
            "visual"
        } else {
            "audio"
        },
        "standard OTIO has no VEAC pipeline; canonical defaults were synthesized",
        false,
    );
    Ok(Clip {
        id,
        enabled: *enabled,
        record_range: TimeRange::new(record_start, range.duration).map_err(OtioError::time)?,
        source: ClipSource::Media {
            material_id: material.id.clone(),
        },
        source_mapping: Some(SourceMapping::linear(
            range.start,
            Rational::new(1, 1).map_err(OtioError::time)?,
        )),
        visual: (track_kind == TrackKind::Video).then(|| super::super::defaults::visual(order)),
        audio: (track_kind == TrackKind::Audio).then(super::super::defaults::audio),
        effects: vec![],
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    })
}

fn resolve_media<'a>(
    reference: &OtioMediaReference,
    media: &BTreeMap<&str, &'a Material>,
    pointer: &str,
) -> Result<(&'a Material, String), OtioError> {
    let OtioMediaReference::External { target_url, .. } = reference else {
        return Err(OtioError::contract(format!(
            "clip {pointer} requires an external media reference"
        )));
    };
    let material = media.get(target_url.as_str()).copied().ok_or_else(|| {
        OtioError::contract(format!("no material binding for target URL {target_url:?}"))
    })?;
    Ok((material, target_url.clone()))
}

fn compatible(kind: MaterialKind, track: TrackKind, pointer: &str) -> Result<(), OtioError> {
    let valid = match track {
        TrackKind::Video => matches!(kind, MaterialKind::Video | MaterialKind::Image),
        TrackKind::Audio => matches!(kind, MaterialKind::Video | MaterialKind::Audio),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(OtioError::contract(format!(
            "material bound to {pointer} is incompatible with its track"
        )))
    }
}
