use veac_plan::canonical::*;
use veac_plan::ResolvedRenderPlan;

use super::super::support::{identity, probe, resolved, text_fixture};

pub(super) fn grouped_plan() -> ResolvedRenderPlan {
    let mut project = text_fixture(true);
    add_media(&mut project, "med_dialogue", 'b');
    add_media(&mut project, "med_music", 'c');
    add_media(&mut project, "med_disabled", 'd');

    let sequence = &mut project.project.sequences[0];
    let base = sequence
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_video")
        .unwrap()
        .clone();
    sequence
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_video")
        .unwrap()
        .state = state(true, true);
    sequence
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .state = state(true, false);
    sequence.tracks.extend([
        audio_track(&base, "dialogue", 5, true, false),
        audio_track(&base, "music", 6, true, true),
        audio_track(&base, "disabled", 7, false, false),
    ]);

    let config = &mut project.project.render_configs[0];
    config.raster.as_mut().unwrap().captions = CaptionOutput::Discard;
    let video = config.deliverables.remove(0);
    config.deliverables = vec![caption(), frames(), video, stem(), scope()];
    config
        .deliverables
        .sort_by(|left, right| left.id.cmp(&right.id));
    resolved(&project)
}

fn add_media(project: &mut ProjectEnvelope, id: &str, fill: char) {
    let mut material = project
        .project
        .materials
        .iter()
        .find(|value| value.id.as_str() == "med_video")
        .unwrap()
        .clone();
    let media_identity = identity(fill);
    material.id = MaterialId::new(id).unwrap();
    material.source = MaterialSource::File {
        uri: format!("media/{id}.mp4"),
    };
    material.identity = Some(media_identity.clone());
    material.probe = Some(probe(media_identity));
    project.project.materials.push(material);
}

fn audio_track(base: &Track, suffix: &str, order: i32, enabled: bool, muted: bool) -> Track {
    let mut track = base.clone();
    track.id = TrackId::new(format!("trk_{suffix}")).unwrap();
    track.kind = TrackKind::Audio;
    track.order = order;
    track.placement_mode = PlacementMode::Free;
    track.state = TrackState {
        enabled,
        muted,
        solo: true,
        locked: false,
    };
    track.clips[0].id = ItemId::new(format!("itm_{suffix}")).unwrap();
    track.clips[0].source = ClipSource::Media {
        material_id: MaterialId::new(format!("med_{suffix}")).unwrap(),
    };
    track.clips[0].visual = None;
    track.clips[0].audio = Some(audio_properties());
    track
}

fn state(solo: bool, muted: bool) -> TrackState {
    TrackState {
        enabled: true,
        muted,
        solo,
        locked: false,
    }
}

fn audio_properties() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}

fn caption() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_caption").unwrap(),
        target: DeliverableTarget::File {
            name: "captions.srt".to_owned(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Srt,
            track_ids: vec![TrackId::new("trk_caption").unwrap()],
        }),
    }
}

fn frames() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_frames").unwrap(),
        target: DeliverableTarget::ImageSequence {
            pattern: "frame-%d.png".to_owned(),
        },
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format: ImageFormat::Png,
            start_number: 1,
        }),
    }
}

fn stem() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "dialogue.wav".to_owned(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Track {
                track_id: TrackId::new("trk_dialogue").unwrap(),
            },
        }),
    }
}

fn scope() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_scope").unwrap(),
        target: DeliverableTarget::File {
            name: "scope.png".to_owned(),
        },
        kind: DeliverableKind::Scope(ScopeOutput {
            scope: VideoScope::Histogram,
            at: RationalTime::zero(600).unwrap(),
            width: 320,
            height: 180,
            format: ImageFormat::Png,
        }),
    }
}
