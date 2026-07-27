use crate::authoring::{
    format_document, lower_document, parse, AnnotationDecl, AnnotationPayloadDecl,
    AnnotationProvenanceDecl, AnnotationTargetDecl, AnnotationTimingDecl, MarkerPayloadDecl, Span,
    Spanned,
};

fn text(value: &str) -> Spanned<String> {
    Spanned {
        value: value.to_owned(),
        span: Span::default(),
    }
}

#[test]
fn a_known_multicam_group_can_be_an_annotation_target_during_lowering() {
    let mut document = parse(super::multicam::PROJECT).unwrap();
    document.project.annotations.push(AnnotationDecl {
        id: text("camera-note"),
        target: AnnotationTargetDecl::Multicam(text("interview")),
        timing: AnnotationTimingDecl::Untimed {
            span: Span::default(),
        },
        payload: AnnotationPayloadDecl::Marker(MarkerPayloadDecl {
            label: text("Preferred angle"),
            color: None,
        }),
        provenance: AnnotationProvenanceDecl {
            producer: text("coverage"),
            request_sha256: text(&"a".repeat(64)),
            response_sha256: text(&"b".repeat(64)),
            span: Span::default(),
        },
        span: Span::default(),
    });
    let formatted = format_document(&document);
    assert!(formatted.contains("target multicam interview;"));
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        envelope.project.annotations[0].target,
        veac_ir::AnnotationTarget::MulticamGroup { ref group_id }
            if group_id.as_str() == "mcg_interview"
    ));
    veac_ir::validate(&envelope).unwrap();
}
