use veac_plan::canonical::{Deliverable, DeliverableId, DeliverableKind, ImageSequencePattern};

use super::super::Check;

pub(super) fn validate(check: &mut Check, values: &[Deliverable]) {
    if values.is_empty() {
        check.push(
            "PLAN_DELIVERABLE_REQUIRED",
            None,
            "at least one deliverable is required",
        );
    }
    if !values.windows(2).all(|pair| pair[0].id < pair[1].id) {
        check.push(
            "PLAN_DELIVERABLE_ORDER_INVALID",
            None,
            "deliverables must be sorted by unique ID",
        );
    }
    for (index, value) in values.iter().enumerate() {
        if DeliverableId::new(value.id.as_str()).is_err() || !name_valid(value) {
            check.push(
                "PLAN_DELIVERABLE_NAME_INVALID",
                Some(value.id.to_string()),
                "deliverable ID or file name is invalid",
            );
        }
        if values[..index].iter().any(|other| overlap(other, value)) {
            check.push(
                "PLAN_DELIVERABLE_COLLISION",
                Some(value.id.to_string()),
                "deliverable file names or image patterns overlap",
            );
        }
    }
}

fn name_valid(value: &Deliverable) -> bool {
    if matches!(value.kind, DeliverableKind::ImageSequence(_)) {
        ImageSequencePattern::parse(&value.file_name).is_some()
            && !value.file_name.is_empty()
            && !value.file_name.contains(['/', '\\', '\0'])
    } else {
        !value.file_name.is_empty()
            && !matches!(value.file_name.as_str(), "." | "..")
            && !value.file_name.contains(['/', '\\', '\0'])
    }
}

enum Name<'a> {
    Static(&'a str),
    Pattern(ImageSequencePattern<'a>),
}

fn overlap(left: &Deliverable, right: &Deliverable) -> bool {
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
