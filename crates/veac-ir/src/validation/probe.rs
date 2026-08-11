use std::collections::BTreeMap;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn probe(&mut self, material: &Material, probe: &MediaProbeSnapshot, path: &str) {
        let probe_path = format!("{path}/probe");
        if probe.schema_version != MEDIA_PROBE_SCHEMA_VERSION
            || probe.engine.trim().is_empty()
            || probe.selection_policy.trim().is_empty()
            || !container_format_valid(&probe.container_format)
            || probe
                .container_brand
                .as_deref()
                .is_some_and(|value| !container_brand_valid(value))
            || probe.observed_identity.algorithm != HashAlgorithm::Sha256
            || !super::values::is_sha256(&probe.observed_identity.digest)
        {
            self.value_error("PROBE_HEADER", &probe_path, material.id.as_str());
        }
        if material
            .identity
            .as_ref()
            .is_some_and(|identity| identity != &probe.observed_identity)
        {
            self.value_error("PROBE_IDENTITY", &probe_path, material.id.as_str());
        }
        self.probe_streams(material, probe, &probe_path);
        let video_valid = selection_valid(
            probe,
            probe.selected_video_stream,
            ProbedStreamType::Video,
            true,
        );
        let audio_valid = selection_valid(
            probe,
            probe.selected_audio_stream,
            ProbedStreamType::Audio,
            false,
        );
        let kind_valid = match material.kind {
            MaterialKind::Video => video_valid,
            MaterialKind::Image => video_valid && probe.selected_audio_stream.is_none(),
            MaterialKind::Audio => audio_valid && probe.selected_video_stream.is_none(),
            MaterialKind::Font | MaterialKind::Lut1d | MaterialKind::Lut3d => false,
        };
        if !kind_valid {
            self.value_error("PROBE_KIND", &probe_path, material.id.as_str());
        }
        if !intent_matches(material.stream_intent.video, probe.selected_video_stream)
            || !intent_matches(material.stream_intent.audio, probe.selected_audio_stream)
        {
            self.value_error("PROBE_INTENT", &probe_path, material.id.as_str());
        }
    }

    fn probe_streams(&mut self, material: &Material, probe: &MediaProbeSnapshot, path: &str) {
        let mut previous_global = None;
        let mut ordinals = BTreeMap::<ProbedStreamType, u32>::new();
        for stream in &probe.streams {
            let expected = ordinals.entry(stream.media_type).or_default();
            if previous_global.is_some_and(|previous| stream.global_index <= previous)
                || stream.type_index != *expected
                || stream.codec.trim().is_empty()
                || stream.time_base.is_some_and(|value| !value.is_positive())
            {
                self.value_error("PROBE_STREAM_ORDER", path, material.id.as_str());
            }
            previous_global = Some(stream.global_index);
            *expected += 1;
            self.stream_times(material, stream, path);
            if !stream_facts_valid(stream) {
                self.value_error("PROBE_STREAM_FACTS", path, material.id.as_str());
            }
        }
    }

    fn stream_times(&mut self, material: &Material, stream: &ProbedStream, path: &str) {
        if let Some(start) = stream.start_time {
            self.intrinsic_time(
                start,
                false,
                "PROBE_STREAM_TIME",
                path,
                material.id.as_str(),
            );
        }
        if let Some(duration) = stream.duration {
            self.intrinsic_time(
                duration,
                true,
                "PROBE_STREAM_TIME",
                path,
                material.id.as_str(),
            );
        }
    }
}

pub fn container_format_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.split(',').all(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
}

fn stream_facts_valid(stream: &ProbedStream) -> bool {
    let typed = match stream.media_type {
        ProbedStreamType::Video => {
            let auxiliary =
                stream.disposition.attached_picture || stream.disposition.timed_thumbnail;
            stream.video.as_ref().is_some_and(|video| {
                input_video_geometry_valid(video)
                    && pixel_format_valid(&video.pixel_format)
                    && video.profile.as_deref().is_none_or(profile_valid)
                    && video.level.is_none_or(|value| value >= 0)
                    && (auxiliary || stream.time_base.is_some())
                    && video_rate_valid(video.frame_rate, auxiliary)
                    && (video.frame_rate.is_some()
                        || (auxiliary && video.cadence == VideoCadence::Unknown))
            }) && stream.audio.is_none()
        }
        ProbedStreamType::Audio => {
            stream.audio.as_ref().is_some_and(input_audio_stream_valid)
                && stream.time_base.is_some()
                && stream.video.is_none()
        }
        _ => stream.video.is_none() && stream.audio.is_none(),
    };
    typed
        && (stream.media_type == ProbedStreamType::Video
            || (!stream.disposition.attached_picture && !stream.disposition.timed_thumbnail))
}

fn container_brand_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
}

fn pixel_format_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn profile_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

fn video_rate_valid(value: Option<Rational>, optional: bool) -> bool {
    match value {
        None => optional,
        Some(value) => {
            value.is_positive()
                && i128::from(value.numerator)
                    <= i128::from(MAX_FRAME_RATE) * i128::from(value.denominator)
        }
    }
}

fn selection_valid(
    probe: &MediaProbeSnapshot,
    selection: Option<StreamSelection>,
    media_type: ProbedStreamType,
    require_playable: bool,
) -> bool {
    selection.is_some_and(|selection| {
        probe.streams.iter().any(|stream| {
            stream.global_index == selection.global_index
                && stream.type_index == selection.type_index
                && stream.media_type == media_type
                && (!require_playable
                    || (!stream.disposition.attached_picture
                        && !stream.disposition.timed_thumbnail))
        })
    })
}

fn intent_matches(intent: StreamChoice, selection: Option<StreamSelection>) -> bool {
    match intent {
        StreamChoice::Auto => true,
        StreamChoice::Disabled => selection.is_none(),
        StreamChoice::GlobalIndex { global_index } => {
            selection.is_some_and(|selection| selection.global_index == global_index)
        }
    }
}
