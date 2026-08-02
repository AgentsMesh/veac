use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::index::syntax;
use crate::program::model::{SlotDecl, SlotKind};

pub(super) fn validate(
    path: &str,
    source: &str,
    slot: &SlotDecl,
    fill_span: Span,
) -> Result<(), Diagnostic> {
    let block = syntax::parse(
        path,
        source,
        Span {
            start: 0,
            end: source.len(),
        },
    )
    .map_err(|_| invalid(path, slot, fill_span))?;
    let [entry] = block.entries.as_slice() else {
        return Err(invalid(path, slot, fill_span));
    };
    let actual = entry
        .word(0)
        .filter(|value| *value == "source")
        .and_then(|_| entry.word(1))
        .and_then(source_kind)
        .ok_or_else(|| invalid(path, slot, fill_span))?;
    if accepts(slot.kind, actual) {
        Ok(())
    } else {
        Err(Diagnostic::new(
            "PROGRAM_SLOT_KIND_MISMATCH",
            path,
            format!("slot `{}` does not accept a `{actual}` source", slot.name),
            fill_span,
        ))
    }
}

fn invalid(path: &str, slot: &SlotDecl, span: Span) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_SLOT_SOURCE",
        path,
        format!(
            "fill for slot `{}` must contain exactly one complete source declaration",
            slot.name
        ),
        span,
    )
}

fn source_kind(value: &str) -> Option<&str> {
    matches!(
        value,
        "media" | "text" | "caption" | "generated" | "sequence" | "multicam"
    )
    .then_some(value)
}

fn accepts(slot: SlotKind, actual: &str) -> bool {
    match slot {
        SlotKind::Video | SlotKind::Audio => actual == "media",
        SlotKind::Visual => matches!(
            actual,
            "media" | "text" | "generated" | "sequence" | "multicam"
        ),
        SlotKind::Text => actual == "text",
        SlotKind::Caption => actual == "caption",
        SlotKind::Sequence => actual == "sequence",
    }
}
