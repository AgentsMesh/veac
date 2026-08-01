use veac_plan::canonical::RationalTime;
use veac_plan::{ResolvedClip, ResolvedClipSource};

use super::{valid_point, values, Curves};

pub(super) fn validate(
    curves: &mut Curves<'_>,
    clip: &ResolvedClip,
    domain: RationalTime,
    owner: &str,
) {
    let content = match &clip.source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            content
        }
        _ => return,
    };
    let Some(animation) = content.styled().and_then(|style| style.animation.as_ref()) else {
        return;
    };
    if !valid_point(animation.stagger, domain, curves.timebase) {
        curves.invalid(owner);
    }
    curves.curve(&animation.reveal, domain, owner, values::unit_number);
    if let Some(highlight) = &animation.highlight {
        curves.curve(&highlight.progress, domain, owner, values::unit_number);
    }
    curves.curve(&animation.opacity, domain, owner, values::unit_number);
    curves.curve(
        &animation.transform.position_offset,
        domain,
        owner,
        values::point,
    );
    curves.curve(
        &animation.transform.scale,
        domain,
        owner,
        values::positive_vec,
    );
    curves.curve(
        &animation.transform.rotation_degrees,
        domain,
        owner,
        values::finite,
    );
}
