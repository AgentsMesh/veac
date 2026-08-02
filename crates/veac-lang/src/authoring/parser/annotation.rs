use crate::authoring::AnnotationDecl;

use super::annotation_fields as fields;
use super::{annotation_payload, annotation_target, Parser};

impl Parser {
    pub(super) fn annotation(&mut self) -> Option<AnnotationDecl> {
        let start = self.required_word("annotation")?;
        let kind = self.identifier("annotation kind")?;
        let id = self.identifier("annotation")?;
        let mut body = self.semantic_block()?;
        let target_entry = fields::required(self, &mut body, "target")?;
        let target = annotation_target::target(self, &target_entry)?;
        let timing_entry = fields::required(self, &mut body, "span")?;
        let timing = annotation_target::timing(self, &timing_entry)?;
        let payload_entry = fields::required(self, &mut body, "payload")?;
        let payload = annotation_payload::parse(self, &kind, &payload_entry)?;
        let provenance_entry = fields::required(self, &mut body, "provenance")?;
        let provenance = annotation_target::provenance(self, &provenance_entry)?;
        let span = start.join(body.span);
        fields::finish(self, body);
        validate_hash(self, "request-sha256", &provenance.request_sha256);
        validate_hash(self, "response-sha256", &provenance.response_sha256);
        Some(AnnotationDecl {
            id,
            target,
            timing,
            payload,
            provenance,
            span,
        })
    }
}

fn validate_hash(parser: &mut Parser, name: &str, value: &crate::authoring::Spanned<String>) {
    let valid = value.value.len() == 64
        && value
            .value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        parser.error(
            "AUTHORING_INVALID_SHA256",
            format!("{name} must be 64 lowercase hexadecimal characters"),
            value.span,
        );
    }
}
