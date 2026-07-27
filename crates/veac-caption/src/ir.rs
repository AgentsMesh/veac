mod metadata;

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    Clip, ClipSource, ItemId, PlacementMode, SequenceId, TextStyle, Track, TrackId, TrackKind,
    TrackRouting, TrackState, VisualProperties,
};

use crate::{
    validate, CaptionCueId, CaptionDocument, CaptionEnvelope, CaptionError, CaptionStyle,
    OverlapPolicy,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionTrackBindings {
    pub track_id: TrackId,
    pub cue_item_ids: BTreeMap<CaptionCueId, ItemId>,
    pub text_style: TextStyle,
    pub visual: VisualProperties,
    pub order: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionTrackInsertionBindings {
    pub sequence_id: SequenceId,
    pub track: CaptionTrackBindings,
    pub before_id: Option<TrackId>,
    pub after_id: Option<TrackId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionDocumentBindings {
    pub cue_ids: BTreeMap<ItemId, CaptionCueId>,
    pub language: Option<String>,
    pub overlap_policy: OverlapPolicy,
    pub settings: BTreeMap<String, String>,
    pub styles: Vec<CaptionStyle>,
}

pub fn to_caption_track(
    value: &CaptionEnvelope,
    bindings: &CaptionTrackBindings,
) -> Result<Track, CaptionError> {
    validate(value)?;
    let mapped: BTreeSet<_> = bindings.cue_item_ids.values().collect();
    if mapped.len() != bindings.cue_item_ids.len() {
        return Err(CaptionError::ir(
            "cue-to-item bindings contain duplicate item IDs",
        ));
    }
    let clips = value
        .document
        .cues
        .iter()
        .map(|cue| {
            let id = bindings.cue_item_ids.get(&cue.id).cloned().ok_or_else(|| {
                CaptionError::ir(format!("missing item ID binding for cue {}", cue.id))
            })?;
            Ok(Clip {
                id,
                enabled: true,
                record_range: cue.range,
                source: ClipSource::Caption {
                    text: cue.text.plain.clone(),
                    speaker: cue.speaker.clone(),
                    style: bindings.text_style.clone(),
                },
                source_mapping: None,
                visual: Some(bindings.visual.clone()),
                audio: None,
                effects: Vec::new(),
                replaceable: None,
                template_editable_text: false,
                metadata: metadata::encode(cue)?,
            })
        })
        .collect::<Result<Vec<_>, CaptionError>>()?;
    Ok(Track {
        id: bindings.track_id.clone(),
        kind: TrackKind::Caption,
        order: bindings.order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    })
}

pub fn from_caption_track(
    track: &Track,
    bindings: &CaptionDocumentBindings,
) -> Result<CaptionEnvelope, CaptionError> {
    if track.kind != TrackKind::Caption {
        return Err(CaptionError::ir("track kind must be caption"));
    }
    let mut cues = Vec::with_capacity(track.clips.len());
    for clip in &track.clips {
        let id = bindings.cue_ids.get(&clip.id).cloned().ok_or_else(|| {
            CaptionError::ir(format!("missing cue ID binding for item {}", clip.id))
        })?;
        let ClipSource::Caption { text, speaker, .. } = &clip.source else {
            return Err(CaptionError::ir(format!(
                "item {} is not a caption clip",
                clip.id
            )));
        };
        cues.push(metadata::decode(
            id,
            clip.record_range,
            text,
            speaker.as_deref(),
            &clip.metadata,
        )?);
    }
    let timescale = cues.first().map_or(1000, |cue| cue.range.start.timescale);
    let value = CaptionEnvelope::new(CaptionDocument {
        timescale,
        language: bindings.language.clone(),
        overlap_policy: bindings.overlap_policy,
        settings: bindings.settings.clone(),
        styles: bindings.styles.clone(),
        cues,
    });
    validate(&value)?;
    Ok(value)
}
