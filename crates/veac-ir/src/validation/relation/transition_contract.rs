use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn transition_contract(
        &mut self,
        transition: &Transition,
        from: RelationItem<'_>,
        to: RelationItem<'_>,
        path: &str,
        id: &RelationId,
    ) {
        self.time(
            transition.duration,
            from.clip.record_range.duration.timescale,
            true,
            "TRANSITION_DURATION",
            &format!("{path}/kind/transition/duration"),
            id.as_str(),
        );
        if !transition_parameters_valid(&transition.kind)
            || transition.duration > from.clip.record_range.duration
        {
            self.value_error("TRANSITION", path, id.as_str());
        }
        if endpoint_blend(from.clip) != BlendMode::Normal
            || endpoint_blend(to.clip) != BlendMode::Normal
        {
            self.value_error("TRANSITION_BLEND_MODE", path, id.as_str());
        }
        if endpoint_z(from.clip) != endpoint_z(to.clip) {
            self.value_error("TRANSITION_Z_ORDER", path, id.as_str());
        }
        let outgoing = outgoing_part(transition);
        let adjacent = from.clip.record_range.end().ok() == Some(to.clip.record_range.start);
        let handles = from.clip.record_range.duration.value >= outgoing
            && to.clip.record_range.duration.value >= transition.duration.value - outgoing;
        if !adjacent || !handles {
            self.value_error("TRANSITION_HANDLES", path, id.as_str());
        }
    }

    pub(super) fn relation_transition_windows(
        &mut self,
        sequences: &[Sequence],
        graph: &RelationGraph<'_>,
    ) {
        for sequence in sequences {
            for track in &sequence.tracks {
                for pair in graph.transitions(&sequence.id, &track.id).windows(2) {
                    let (incoming, outgoing) = (pair[0], pair[1]);
                    if incoming.to.clip.id != outgoing.from.clip.id {
                        continue;
                    }
                    let incoming_end = incoming.to.clip.record_range.start.value.checked_add(
                        incoming.transition.duration.value - outgoing_part(incoming.transition),
                    );
                    let outgoing_start = outgoing
                        .to
                        .clip
                        .record_range
                        .start
                        .value
                        .checked_sub(outgoing_part(outgoing.transition));
                    if incoming_end
                        .zip(outgoing_start)
                        .is_some_and(|(end, start)| end > start)
                    {
                        let path = format!("/project/relations/{}/kind", outgoing.relation_id);
                        self.value_error(
                            "TRANSITION_WINDOW_OVERLAP",
                            &path,
                            outgoing.relation_id.as_str(),
                        );
                    }
                }
            }
        }
    }
}

fn outgoing_part(value: &Transition) -> i64 {
    match value.alignment {
        TransitionAlignment::BeforeCut => value.duration.value,
        TransitionAlignment::Centered => value.duration.value / 2,
        TransitionAlignment::AfterCut => 0,
    }
}

fn endpoint_blend(clip: &Clip) -> BlendMode {
    clip.visual
        .as_ref()
        .map_or(BlendMode::Normal, |visual| visual.compositing.blend_mode)
}

fn endpoint_z(clip: &Clip) -> i32 {
    clip.visual
        .as_ref()
        .map_or(0, |visual| visual.compositing.z_index)
}
