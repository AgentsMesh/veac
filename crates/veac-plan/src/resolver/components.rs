use crate::ResolvedSequence;

#[derive(Clone, Copy)]
pub(super) struct Components {
    pub video: bool,
    pub audio: bool,
}

pub(super) fn sequence(sequence: &ResolvedSequence) -> Components {
    let video = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .any(|clip| clip.visual.is_some());
    let audio = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .any(|clip| {
            clip.audio
                .as_ref()
                .is_some_and(|properties| !properties.muted)
        });
    Components { video, audio }
}
