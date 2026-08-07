mod window;

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
        if !transition_parameters_valid(&transition.kind) {
            self.value_error("TRANSITION", path, id.as_str());
        }
        if !matches!(from.track.kind, TrackKind::Video | TrackKind::Visual) {
            self.value_error("TRANSITION_TRACK_TYPE", path, id.as_str());
        }
        if from.clip.visual.is_none() || to.clip.visual.is_none() {
            self.value_error("TRANSITION_VISUAL_ENDPOINT", path, id.as_str());
        }
        if endpoint_blend(from.clip) != BlendMode::Normal
            || endpoint_blend(to.clip) != BlendMode::Normal
        {
            self.value_error("TRANSITION_BLEND_MODE", path, id.as_str());
        }
        if endpoint_z(from.clip) != endpoint_z(to.clip) {
            self.value_error("TRANSITION_Z_ORDER", path, id.as_str());
        }
        let Some(record_window) = window::exact(from.clip, to.clip) else {
            self.value_error("TRANSITION_OVERLAP", path, id.as_str());
            return;
        };
        if transition.duration != record_window.duration {
            self.value_error("TRANSITION_DURATION_MISMATCH", path, id.as_str());
        }
        if window::third_item_crosses(from, to, record_window) {
            self.value_error("TRANSITION_THIRD_ITEM", path, id.as_str());
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
                    let windows = window::exact(incoming.from.clip, incoming.to.clip)
                        .zip(window::exact(outgoing.from.clip, outgoing.to.clip));
                    if windows.is_some_and(|(left, right)| window::intersects(left, right)) {
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
