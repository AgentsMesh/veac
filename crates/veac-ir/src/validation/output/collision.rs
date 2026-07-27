use crate::{Deliverable, DeliverableKind, ImageSequencePattern};

enum Name<'a> {
    Static(&'a str),
    Pattern(ImageSequencePattern<'a>),
}

pub(super) fn overlap(left: &Deliverable, right: &Deliverable) -> bool {
    match (name(left), name(right)) {
        (Name::Static(left), Name::Static(right)) => left == right,
        (Name::Static(value), Name::Pattern(pattern))
        | (Name::Pattern(pattern), Name::Static(value)) => pattern.matches(value),
        (Name::Pattern(left), Name::Pattern(right)) => left.overlaps(right),
    }
}

fn name(value: &Deliverable) -> Name<'_> {
    match &value.kind {
        DeliverableKind::ImageSequence(_) => ImageSequencePattern::parse(&value.file_name)
            .map(Name::Pattern)
            .unwrap_or(Name::Static(&value.file_name)),
        _ => Name::Static(&value.file_name),
    }
}
