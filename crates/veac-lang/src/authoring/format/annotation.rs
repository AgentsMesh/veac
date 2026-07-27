use crate::authoring::{
    AnnotationDecl, AnnotationPayloadDecl, AnnotationTargetDecl, AnnotationTimingDecl,
};

use super::annotation_payload::payload;
use super::value::quoted;
use super::writer::Writer;

pub(super) fn annotation(writer: &mut Writer, value: &AnnotationDecl) {
    writer.block(
        format!("annotation {} {}", kind(&value.payload), value.id.value),
        |writer| {
            writer.line(format!("target {};", target(&value.target)));
            timing(writer, &value.timing);
            payload(writer, &value.payload);
            writer.block("provenance", |writer| {
                writer.line(format!(
                    "producer {};",
                    quoted(&value.provenance.producer.value)
                ));
                writer.line(format!(
                    "request-sha256 {};",
                    quoted(&value.provenance.request_sha256.value)
                ));
                writer.line(format!(
                    "response-sha256 {};",
                    quoted(&value.provenance.response_sha256.value)
                ));
            });
        },
    );
}

fn kind(value: &AnnotationPayloadDecl) -> &'static str {
    match value {
        AnnotationPayloadDecl::Marker(_) => "marker",
        AnnotationPayloadDecl::Language(_) => "language",
        AnnotationPayloadDecl::SceneBoundary(_) => "scene-boundary",
        AnnotationPayloadDecl::Scene => "scene",
        AnnotationPayloadDecl::Beat(_) => "beat",
        AnnotationPayloadDecl::Silence(_) => "silence",
        AnnotationPayloadDecl::Filler(_) => "filler",
        AnnotationPayloadDecl::Highlight(_) => "highlight",
        AnnotationPayloadDecl::Review(_) => "review",
    }
}

fn target(value: &AnnotationTargetDecl) -> String {
    match value {
        AnnotationTargetDecl::Project { .. } => "project".to_owned(),
        AnnotationTargetDecl::Sequence(id) => format!("sequence {}", id.value),
        AnnotationTargetDecl::Layer(id) => format!("layer {}", id.value),
        AnnotationTargetDecl::Item(id) => format!("item {}", id.value),
        AnnotationTargetDecl::Resource(id) => format!("resource {}", id.value),
        AnnotationTargetDecl::Multicam(id) => format!("multicam {}", id.value),
    }
}

fn timing(writer: &mut Writer, value: &AnnotationTimingDecl) {
    match value {
        AnnotationTimingDecl::Untimed { .. } => writer.line("span untimed;"),
        AnnotationTimingDecl::Point { at, .. } => writer.block("span point", |writer| {
            writer.line(format!("at {};", at.raw));
        }),
        AnnotationTimingDecl::Range { at, duration, .. } => {
            writer.block("span range", |writer| {
                writer.line(format!("at {};", at.raw));
                writer.line(format!("duration {};", duration.raw));
            });
        }
    }
}
