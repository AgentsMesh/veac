use crate::{Deliverable, DeliverableTarget, ImageSequencePattern};

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
    match &value.target {
        DeliverableTarget::ImageSequence { pattern } => ImageSequencePattern::parse(pattern)
            .map(Name::Pattern)
            .unwrap_or(Name::Static(pattern)),
        DeliverableTarget::File { name } | DeliverableTarget::Package { name } => {
            Name::Static(name)
        }
    }
}
