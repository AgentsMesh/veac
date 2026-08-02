use crate::authoring::{
    AnnotationPayloadDecl, BeatPayloadDecl, FillerSuggestionDecl, ReviewActionDecl,
};

use super::value::quoted;
use super::writer::Writer;

pub(super) fn payload(writer: &mut Writer, value: &AnnotationPayloadDecl) {
    writer.block("payload", |writer| match value {
        AnnotationPayloadDecl::Marker(value) => {
            writer.line(format!("label {};", quoted(&value.label.value)));
            if let Some(color) = &value.color {
                writer.line(format!("color {};", color.value));
            }
        }
        AnnotationPayloadDecl::Language(value) => {
            for candidate in &value.candidates {
                writer.block(
                    format!("candidate {}", quoted(&candidate.language.value)),
                    |writer| {
                        writer.line(format!("confidence {};", candidate.confidence.raw));
                    },
                );
            }
        }
        AnnotationPayloadDecl::SceneBoundary(value) => {
            writer.line(format!("confidence {};", value.confidence.raw));
            writer.line(format!("hard-cut {};", value.hard_cut.value));
        }
        AnnotationPayloadDecl::Scene => {}
        AnnotationPayloadDecl::Beat(value) => beat(writer, value),
        AnnotationPayloadDecl::Silence(value) => {
            writer.line(format!("mean-db {};", value.mean_db.raw));
            writer.line(format!("confidence {};", value.confidence.raw));
        }
        AnnotationPayloadDecl::Filler(value) => {
            writer.line(format!("token {};", quoted(&value.token.value)));
            writer.line(format!("confidence {};", value.confidence.raw));
            writer.line(format!(
                "suggestion {};",
                match value.suggestion.value {
                    FillerSuggestionDecl::Keep => "keep",
                    FillerSuggestionDecl::Delete => "delete",
                    FillerSuggestionDecl::Tighten => "tighten",
                }
            ));
        }
        AnnotationPayloadDecl::Highlight(value) => {
            writer.line(format!("score {};", value.score.raw));
            writer.line(format!("rationale {};", quoted(&value.rationale.value)));
            for evidence in &value.evidence {
                writer.line(format!("evidence {};", quoted(&evidence.value)));
            }
        }
        AnnotationPayloadDecl::Review(value) => {
            writer.line(format!(
                "action {};",
                match value.action.value {
                    ReviewActionDecl::Keep => "keep",
                    ReviewActionDecl::Remove => "remove",
                }
            ));
            writer.line(format!("rationale {};", quoted(&value.rationale.value)));
            writer.line(format!("confidence {};", value.confidence.raw));
        }
    });
}

fn beat(writer: &mut Writer, value: &BeatPayloadDecl) {
    writer.line(format!("confidence {};", value.confidence.raw));
    writer.line(format!("bar {};", value.bar.raw));
    writer.line(format!("beat-in-bar {};", value.beat_in_bar.raw));
    writer.line(format!("tempo-bpm {};", value.tempo_bpm.raw));
    writer.line(format!("meter {};", value.meter.raw));
}
