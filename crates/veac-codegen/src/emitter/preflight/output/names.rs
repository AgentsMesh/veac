use veac_plan::canonical::{
    Deliverable, DeliverableId, DeliverableKind, DeliverableTarget, ImageSequencePattern,
};

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
    match (&value.kind, &value.target) {
        (DeliverableKind::ImageSequence(_), DeliverableTarget::ImageSequence { pattern }) => {
            ImageSequencePattern::parse(pattern).is_some()
        }
        (
            DeliverableKind::Video(_)
            | DeliverableKind::CaptionSidecar(_)
            | DeliverableKind::AudioStem(_)
            | DeliverableKind::Scope(_)
            | DeliverableKind::AudioFile(_)
            | DeliverableKind::AnimatedImage(_)
            | DeliverableKind::StillImage(_),
            DeliverableTarget::File { name },
        ) => {
            !name.is_empty()
                && !matches!(name.as_str(), "." | "..")
                && !name.contains(['/', '\\', '\0'])
        }
        (DeliverableKind::AdaptivePackage(_), DeliverableTarget::Package { name }) => {
            leaf_name(name)
        }
        _ => false,
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
    match &value.target {
        DeliverableTarget::ImageSequence { pattern } => ImageSequencePattern::parse(pattern)
            .map(Name::Pattern)
            .unwrap_or(Name::Static(pattern)),
        DeliverableTarget::File { name } | DeliverableTarget::Package { name } => {
            Name::Static(name)
        }
    }
}

fn leaf_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !matches!(name, "." | "..")
        && !name
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
}
