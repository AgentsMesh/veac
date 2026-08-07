use sha2::{Digest, Sha256};

use super::{error, ExecutableLowerError, ExecutableTemporalSink, FrozenTemporalAttachment};
use crate::program::expression::{CoreDigest, ResidualizationLimits, ResidualizationRequest};
use veac_ir::{
    TemporalAuthoredSite, TemporalBindingId, TemporalDefinitionId, TemporalDefinitionKind,
    TemporalDefinitionSite, TemporalLogicalKey, TemporalProgramId, TemporalProvenance,
    TemporalProvenanceId, TemporalSourceId, TemporalSourceSpan, MAX_TEMPORAL_CALL_DEPTH,
};

pub(super) fn binding(sink: &ExecutableTemporalSink, definition: CoreDigest) -> TemporalBindingId {
    TemporalBindingId::new(sink_definition_id(
        "tbd_",
        b"binding",
        sink,
        &definition.to_string(),
    ))
    .expect("stable binding ID")
}

pub(super) fn request(
    sink: &ExecutableTemporalSink,
    attachment: FrozenTemporalAttachment<'_>,
    content: CoreDigest,
) -> Result<ResidualizationRequest, ExecutableLowerError> {
    let definition = attachment.animation().provenance().ok_or_else(|| {
        error(
            "EXECUTABLE_TEMPORAL_PROVENANCE",
            "animation closure has no authored declaration",
        )
    })?;
    let instance = attachment.instance().ok_or_else(|| {
        error(
            "EXECUTABLE_TEMPORAL_PROVENANCE",
            "animation attachment has no authored instance",
        )
    })?;
    let definition = definition.temporal_view();
    let instance = instance.temporal_view();
    let definition_id = definition_id(&definition);
    let program = sink_content_id("tpg_", b"program", sink, &definition.identity, content);
    let provenance = sink_content_id("tpv_", b"provenance", sink, &definition.identity, content);
    let logical_key = sink_content_id("key_", b"logical-key", sink, &definition.identity, content);
    let definition = TemporalDefinitionSite {
        id: definition_id,
        kind: if definition.closure {
            TemporalDefinitionKind::Closure
        } else {
            TemporalDefinitionKind::Function
        },
        name: definition.name,
        source_id: source_id(&definition.source),
        span: span(definition.span)?,
    };
    let origin = TemporalAuthoredSite {
        definition_id: definition.id.clone(),
        function: definition.name.clone(),
        source_id: definition.source_id.clone(),
        span: definition.span,
    };
    let mut call_stack = vec![authored_site(instance.origin)?];
    call_stack.extend(
        instance
            .call_stack
            .into_iter()
            .take(MAX_TEMPORAL_CALL_DEPTH.saturating_sub(1))
            .map(authored_site)
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(ResidualizationRequest {
        program_id: TemporalProgramId::new(program).expect("stable program ID"),
        provenance: TemporalProvenance {
            id: TemporalProvenanceId::new(provenance).expect("stable provenance ID"),
            definition,
            origin,
            call_stack,
            logical_keys: vec![TemporalLogicalKey::new(logical_key).expect("stable logical key")],
        },
        limits: ResidualizationLimits::default(),
    })
}

fn authored_site(
    value: crate::program::expression::provenance::TemporalSiteView,
) -> Result<TemporalAuthoredSite, ExecutableLowerError> {
    Ok(TemporalAuthoredSite {
        definition_id: text_definition_id(&value.definition),
        function: value.function,
        source_id: source_id(&value.source),
        span: span(value.span)?,
    })
}

fn definition_id(
    value: &crate::program::expression::provenance::TemporalDefinitionView,
) -> TemporalDefinitionId {
    text_definition_id(&value.identity)
}

fn text_definition_id(value: &str) -> TemporalDefinitionId {
    TemporalDefinitionId::new(text_id("def_", b"definition", value)).expect("stable definition ID")
}

fn source_id(value: &str) -> TemporalSourceId {
    TemporalSourceId::new(text_id("src_", b"source", value)).expect("stable source ID")
}

fn span(value: std::ops::Range<usize>) -> Result<TemporalSourceSpan, ExecutableLowerError> {
    Ok(TemporalSourceSpan {
        start: u32::try_from(value.start).map_err(|_| span_error())?,
        end: u32::try_from(value.end).map_err(|_| span_error())?,
    })
}

fn span_error() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_PROVENANCE",
        "authored source span exceeds u32",
    )
}

fn sink_definition_id(
    prefix: &str,
    domain: &[u8],
    sink: &ExecutableTemporalSink,
    definition: &str,
) -> String {
    let mut digest = sink_digest(domain, sink);
    frame(&mut digest, definition.as_bytes());
    format!("{prefix}{:x}", digest.finalize())
}

fn sink_content_id(
    prefix: &str,
    domain: &[u8],
    sink: &ExecutableTemporalSink,
    definition: &str,
    content: CoreDigest,
) -> String {
    let mut digest = sink_digest(domain, sink);
    frame(&mut digest, definition.as_bytes());
    frame(&mut digest, content.as_bytes());
    format!("{prefix}{:x}", digest.finalize())
}

fn sink_digest(domain: &[u8], sink: &ExecutableTemporalSink) -> Sha256 {
    let mut digest = Sha256::new();
    digest.update(b"veac.component-temporal.v1\0");
    frame(&mut digest, domain);
    frame(&mut digest, &sink.identity_bytes());
    digest
}

fn text_id(prefix: &str, domain: &[u8], value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"veac.component-temporal.source.v1\0");
    frame(&mut digest, domain);
    frame(&mut digest, value.as_bytes());
    format!("{prefix}{:x}", digest.finalize())
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
mod tests;
