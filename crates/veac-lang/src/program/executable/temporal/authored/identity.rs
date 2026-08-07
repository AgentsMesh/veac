use sha2::{Digest, Sha256};

use crate::program::expression::{ResidualizationLimits, ResidualizationRequest};
use crate::program::model::{SurfaceFile, TemporalDecl, TemporalProperty, TemporalTarget};
use veac_ir::{
    TemporalAuthoredSite, TemporalBindingId, TemporalDefinitionId, TemporalDefinitionKind,
    TemporalDefinitionSite, TemporalLogicalKey, TemporalProgramId, TemporalProvenance,
    TemporalProvenanceId, TemporalSourceId, TemporalSourceSpan,
};

pub(super) fn binding(path: &TemporalTarget, property: TemporalProperty) -> TemporalBindingId {
    TemporalBindingId::new(id("tbd_", b"binding", path, property)).expect("stable binding ID")
}

pub(super) fn request(file: &SurfaceFile, value: &TemporalDecl) -> ResidualizationRequest {
    let source_id = TemporalSourceId::new(source_id(&file.path)).expect("stable source ID");
    let definition_id =
        TemporalDefinitionId::new(id("def_", b"definition", &value.target, value.property))
            .expect("stable definition ID");
    let span = source_span(value.body.span);
    let authored = TemporalAuthoredSite {
        definition_id: definition_id.clone(),
        function: "animate".to_owned(),
        source_id: source_id.clone(),
        span,
    };
    ResidualizationRequest {
        program_id: TemporalProgramId::new(id("tpg_", b"program", &value.target, value.property))
            .expect("stable program ID"),
        provenance: TemporalProvenance {
            id: TemporalProvenanceId::new(id("tpv_", b"provenance", &value.target, value.property))
                .expect("stable provenance ID"),
            definition: TemporalDefinitionSite {
                id: definition_id,
                kind: TemporalDefinitionKind::Closure,
                name: property_name(value.property).to_owned(),
                source_id,
                span: source_span(value.span),
            },
            origin: authored,
            call_stack: Vec::new(),
            logical_keys: vec![TemporalLogicalKey::new(id(
                "key_",
                b"logical-key",
                &value.target,
                value.property,
            ))
            .expect("stable logical key")],
        },
        limits: ResidualizationLimits::default(),
    }
}

fn id(prefix: &str, domain: &[u8], path: &TemporalTarget, property: TemporalProperty) -> String {
    let mut digest = Sha256::new();
    digest.update(b"veac.authored-temporal.v1\0");
    frame(&mut digest, domain);
    target(&mut digest, path);
    frame(&mut digest, property.source_name().as_bytes());
    format!("{prefix}{:x}", digest.finalize())
}

fn source_id(path: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"veac.authored-temporal.source.v1\0");
    frame(&mut digest, path.as_bytes());
    format!("src_{:x}", digest.finalize())
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn source_span(span: crate::authoring::Span) -> TemporalSourceSpan {
    TemporalSourceSpan {
        start: u32::try_from(span.start).expect("source budget keeps spans within u32"),
        end: u32::try_from(span.end).expect("source budget keeps spans within u32"),
    }
}

fn property_name(value: TemporalProperty) -> &'static str {
    value.source_name()
}

fn target(digest: &mut Sha256, value: &TemporalTarget) {
    match value {
        TemporalTarget::Clip(path) => segments(digest, &path.segments()),
        TemporalTarget::Text(path) => tagged(digest, b"text", &path.segments()),
        TemporalTarget::ClipMask { clip, mask_index } => {
            tagged(digest, b"clip-mask", &clip.segments());
            frame(digest, &mask_index.to_be_bytes());
        }
        TemporalTarget::ClipEffect {
            clip,
            effect,
            parameter,
        } => {
            tagged(digest, b"clip-effect", &clip.segments());
            frame(digest, effect.as_bytes());
            frame(digest, parameter.as_bytes());
        }
        TemporalTarget::Apply(path) => tagged(digest, b"apply", &path.segments()),
        TemporalTarget::ApplyMask { apply, mask_index } => {
            tagged(digest, b"apply-mask", &apply.segments());
            frame(digest, &mask_index.to_be_bytes());
        }
        TemporalTarget::ApplyEffect {
            apply,
            stage,
            effect,
            parameter,
        } => {
            tagged(digest, b"apply-effect", &apply.segments());
            frame(digest, stage.as_bytes());
            frame(digest, effect.as_bytes());
            frame(digest, parameter.as_bytes());
        }
    }
}

fn tagged(digest: &mut Sha256, tag: &[u8], values: &[&str]) {
    frame(digest, tag);
    segments(digest, values);
}

fn segments(digest: &mut Sha256, values: &[&str]) {
    for value in values {
        frame(digest, value.as_bytes());
    }
}
