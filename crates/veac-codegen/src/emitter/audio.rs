use std::collections::BTreeMap;

use veac_plan::canonical::{AudioMixSource, AudioOutput, TrackKind};
use veac_plan::{ResolvedAudioRoute, ResolvedSequence, ResolvedTrack};

use super::{
    audio_sidechain, audio_source, audio_transition_fades, time, CodegenErrors, EmitContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AudioRenderSpec {
    pub sample_rate: u32,
    pub channels: u8,
}

impl From<&AudioOutput> for AudioRenderSpec {
    fn from(value: &AudioOutput) -> Self {
        Self {
            sample_rate: value.sample_rate,
            channels: value.channels,
        }
    }
}

pub(super) fn build_audio(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    output: &AudioOutput,
) -> Result<String, CodegenErrors> {
    let output = AudioRenderSpec::from(output);
    build_master(context, sequence, &output)
}

pub(super) fn build_master(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    output: &AudioRenderSpec,
) -> Result<String, CodegenErrors> {
    let mut routes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for track in &sequence.tracks {
        if !track.state.audio_enabled || !matches!(track.kind, TrackKind::Video | TrackKind::Audio)
        {
            continue;
        }
        let route = match track.routing.audio.as_ref() {
            Some(ResolvedAudioRoute::MainMix) | None => "main".to_owned(),
            Some(ResolvedAudioRoute::Bus { bus_id }) => format!("bus:{bus_id}"),
        };
        routes
            .entry(route)
            .or_default()
            .extend(build_track(context, sequence, track, output)?);
    }
    let labels: Vec<_> = routes
        .into_values()
        .filter(|labels| !labels.is_empty())
        .map(|labels| route_mix(context, &labels, output))
        .collect();
    Ok(finish(context, sequence, output, labels))
}

pub(super) fn build_stem(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    source: &AudioMixSource,
    output: AudioRenderSpec,
) -> Result<String, CodegenErrors> {
    if *source == AudioMixSource::Master {
        return build_master(context, sequence, &output);
    }
    let mut labels = Vec::new();
    for track in &sequence.tracks {
        let selected = match source {
            AudioMixSource::Track { track_id } => track.id == *track_id,
            AudioMixSource::Bus { bus_id } => matches!(
                &track.routing.audio,
                Some(ResolvedAudioRoute::Bus { bus_id: value }) if value == bus_id.as_str()
            ),
            AudioMixSource::Master => false,
        };
        if selected && track.state.audio_enabled {
            labels.extend(build_track(context, sequence, track, &output)?);
        }
    }
    Ok(finish(context, sequence, &output, labels))
}

fn finish(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    output: &AudioRenderSpec,
    mut labels: Vec<String>,
) -> String {
    labels.push(silence(context, sequence, output));
    let mixed = mix(context, &labels, "mix");
    context.graph.filter(
        &[&mixed],
        format!(
            "atrim=duration={},asetpts=PTS-STARTPTS,aresample={}",
            time::seconds(sequence.duration),
            output.sample_rate
        ),
        "audioout",
    )
}

pub(super) fn mix(context: &mut EmitContext<'_>, labels: &[String], prefix: &str) -> String {
    if labels.len() == 1 {
        return labels[0].clone();
    }
    let refs: Vec<_> = labels.iter().map(String::as_str).collect();
    context.graph.filter(
        &refs,
        format!(
            "amix=inputs={}:normalize=0:dropout_transition=0",
            refs.len()
        ),
        prefix,
    )
}

fn route_mix(context: &mut EmitContext<'_>, labels: &[String], output: &AudioRenderSpec) -> String {
    let mixed = mix(context, labels, "busmix");
    context.graph.filter(
        &[&mixed],
        format!(
            "aresample={},asetnsamples=n=1024:p=1,asetpts=N/SR/TB",
            output.sample_rate
        ),
        "busout",
    )
}

fn build_track(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    track: &ResolvedTrack,
    output: &AudioRenderSpec,
) -> Result<Vec<String>, CodegenErrors> {
    let mut labels = Vec::new();
    for clip in &track.clips {
        if !audio_source::can_build(clip) {
            continue;
        }
        let fades = audio_transition_fades::TransitionFades {
            fade_in: track
                .transitions
                .iter()
                .find(|transition| transition.incoming_clip_id == clip.id)
                .map(|transition| transition.incoming_handle.duration),
            fade_out: track
                .transitions
                .iter()
                .find(|transition| transition.outgoing_clip_id == clip.id)
                .map(|transition| transition.outgoing_handle.duration),
        };
        let sidechain = match clip
            .audio
            .as_ref()
            .and_then(|audio| audio.sidechain.as_ref())
        {
            Some(value) => audio_sidechain::source(context, sequence, track, clip, value, output)?,
            None => None,
        };
        if let Some(label) = audio_source::build(context, clip, output, fades, sidechain)? {
            labels.push(label);
        }
    }
    Ok(labels)
}

fn silence(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    output: &AudioRenderSpec,
) -> String {
    context.graph.source(
        format!(
            "anullsrc=r={}:cl={},atrim=duration={},asetpts=PTS-STARTPTS",
            output.sample_rate,
            channel_layout(output.channels),
            time::seconds(sequence.duration)
        ),
        "silence",
    )
}

pub(crate) fn channel_layout(channels: u8) -> String {
    match channels {
        1 => "mono".to_owned(),
        2 => "stereo".to_owned(),
        value => format!("{value}c"),
    }
}
