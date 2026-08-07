use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{ClipSource, RationalTime};

use super::super::error::ExecutableLowerError;
use super::super::{text, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    duration: RationalTime,
    scope: &[&str],
) -> Result<ClipSource, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SourceText, [content, style]) => {
            let content = value::text(Some(content))?;
            Ok(ClipSource::Text {
                text: content.to_owned(),
                style: text::style(graph, style, content, timebase, duration, scope)?,
            })
        }
        (Op::SourceCaption, [content, style]) => {
            let content = value::text(Some(content))?;
            Ok(ClipSource::Caption {
                text: content.to_owned(),
                speaker: None,
                cue: Box::new(veac_ir::CaptionCueSemantics::default()),
                style: text::style(graph, style, content, timebase, duration, scope)?,
            })
        }
        (Op::SourceCaptionSpeaker, [content, speaker, style]) => {
            let content = value::text(Some(content))?;
            let speaker = value::text(Some(speaker))?;
            if speaker.trim().is_empty()
                || speaker.chars().count() > 256
                || speaker
                    .chars()
                    .any(|value| matches!(value, '\0' | '\r' | '\n'))
            {
                return Err(text::invalid());
            }
            Ok(ClipSource::Caption {
                text: content.to_owned(),
                speaker: Some(speaker.to_owned()),
                cue: Box::new(veac_ir::CaptionCueSemantics::default()),
                style: text::style(graph, style, content, timebase, duration, scope)?,
            })
        }
        _ => Err(text::invalid()),
    }
}
