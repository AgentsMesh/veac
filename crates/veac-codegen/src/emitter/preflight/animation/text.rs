use veac_plan::canonical::RationalTime;
use veac_plan::{ResolvedClip, ResolvedClipSource};

use crate::emitter::process_owner::ProcessOwner;

use super::{range, values, Curves};

pub(super) fn validate(
    curves: &mut Curves<'_>,
    clip: &ResolvedClip,
    domain: RationalTime,
    owner: &str,
    process: ProcessOwner<'_>,
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
    if !range::point(animation.stagger, domain, curves.timebase) {
        curves.invalid(owner);
    }
    curves.curve(
        &animation.reveal,
        domain,
        owner,
        process,
        values::unit_number,
    );
    if let Some(highlight) = &animation.highlight {
        curves.curve(
            &highlight.progress,
            domain,
            owner,
            process,
            values::unit_number,
        );
    }
    curves.curve(
        &animation.opacity,
        domain,
        owner,
        process,
        values::unit_number,
    );
    curves.curve(
        &animation.transform.position_offset,
        domain,
        owner,
        process,
        values::point,
    );
    curves.curve(
        &animation.transform.scale,
        domain,
        owner,
        process,
        values::positive_vec,
    );
    curves.curve(
        &animation.transform.rotation_degrees,
        domain,
        owner,
        process,
        values::finite,
    );
}
