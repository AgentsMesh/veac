use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use veac_ir::TimeRange;

use crate::{CaptionCue, CaptionCueId, CaptionError, CaptionSpan, CaptionText, CaptionWord};

const KEY: &str = "veac.caption.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CueMetadata {
    spans: Vec<CaptionSpan>,
    style: Option<String>,
    settings: BTreeMap<String, String>,
    words: Vec<CaptionWord>,
}

pub(super) fn encode(cue: &CaptionCue) -> Result<BTreeMap<String, Value>, CaptionError> {
    let extension = CueMetadata {
        spans: cue.text.spans.clone(),
        style: cue.style.clone(),
        settings: cue.settings.clone(),
        words: cue.words.clone(),
    };
    let value = serde_json::to_value(extension).map_err(CaptionError::Json)?;
    Ok(BTreeMap::from([(KEY.to_owned(), value)]))
}

pub(super) fn decode(
    id: CaptionCueId,
    range: TimeRange,
    text: &str,
    speaker: Option<&str>,
    metadata: &BTreeMap<String, Value>,
) -> Result<CaptionCue, CaptionError> {
    let extension = metadata.get(KEY).cloned().map_or_else(
        || {
            Ok(CueMetadata {
                spans: Vec::new(),
                style: None,
                settings: BTreeMap::new(),
                words: Vec::new(),
            })
        },
        |value| serde_json::from_value(value).map_err(CaptionError::Json),
    )?;
    Ok(CaptionCue {
        id,
        range,
        text: CaptionText {
            plain: text.to_owned(),
            spans: extension.spans,
        },
        speaker: speaker.map(str::to_owned),
        style: extension.style,
        settings: extension.settings,
        words: extension.words,
    })
}
