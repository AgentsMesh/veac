use crate::{InstanceSelector, IssueCode, TargetInstance, TargetRef};

pub(crate) fn select(
    reference: &TargetRef,
    consumer: &TargetInstance,
    candidates: &[TargetInstance],
) -> Result<Vec<TargetInstance>, (IssueCode, String)> {
    let mut selected: Vec<_> = candidates
        .iter()
        .filter(|candidate| profile_matches(reference, consumer, candidate))
        .filter(|candidate| selector_matches(&reference.selector, consumer, candidate))
        .cloned()
        .collect();
    selected.sort_by(|left, right| left.id.cmp(&right.id));
    let permits_many = matches!(reference.selector, InstanceSelector::AllMatching {});
    match selected.len() {
        0 => Err((
            IssueCode::SelectorNoMatch,
            format!(
                "reference to {:?} selects no instance",
                reference.target.as_str()
            ),
        )),
        1 => Ok(selected),
        _ if permits_many => Ok(selected),
        count => Err((
            IssueCode::SelectorNotUnique,
            format!(
                "reference to {:?} selects {count} instances; use exact or all_matching",
                reference.target.as_str()
            ),
        )),
    }
}

fn profile_matches(
    reference: &TargetRef,
    consumer: &TargetInstance,
    candidate: &TargetInstance,
) -> bool {
    if let Some(profile) = &reference.profile {
        return candidate.profile.as_ref() == Some(profile);
    }
    match reference.selector {
        InstanceSelector::Same {} => candidate
            .profile
            .as_ref()
            .is_none_or(|profile| consumer.profile.as_ref() == Some(profile)),
        InstanceSelector::Exact { .. } => true,
        InstanceSelector::AllMatching {} => consumer.profile.as_ref().is_none_or(|profile| {
            candidate
                .profile
                .as_ref()
                .is_none_or(|value| value == profile)
        }),
    }
}

fn selector_matches(
    selector: &InstanceSelector,
    consumer: &TargetInstance,
    candidate: &TargetInstance,
) -> bool {
    match selector {
        InstanceSelector::Same {} => same(consumer, candidate),
        InstanceSelector::Exact { locale, axes } => {
            candidate.locale == *locale
                && axes
                    .iter()
                    .all(|(axis, value)| candidate.matrix.get(axis) == Some(value))
        }
        InstanceSelector::AllMatching {} => shared(consumer, candidate),
    }
}

fn same(consumer: &TargetInstance, candidate: &TargetInstance) -> bool {
    candidate
        .locale
        .as_ref()
        .is_none_or(|locale| consumer.locale.as_ref() == Some(locale))
        && candidate
            .matrix
            .iter()
            .all(|(axis, value)| consumer.matrix.get(axis) == Some(value))
}

fn shared(consumer: &TargetInstance, candidate: &TargetInstance) -> bool {
    let locale_matches = consumer
        .locale
        .as_ref()
        .zip(candidate.locale.as_ref())
        .is_none_or(|(left, right)| left == right);
    locale_matches
        && candidate.matrix.iter().all(|(axis, value)| {
            consumer
                .matrix
                .get(axis)
                .is_none_or(|current| current == value)
        })
}
