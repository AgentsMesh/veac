mod caption;

use crate::*;

use super::super::Validator;

impl Validator {
    pub(super) fn aux_deliverable(
        &mut self,
        value: &Deliverable,
        project: &Project,
        output: &RenderConfig,
        path: &str,
    ) {
        let sequence_id = &output.sequence_id;
        match &value.kind {
            DeliverableKind::ImageSequence(settings) => {
                if !image_pattern_valid(value)
                    || !image_extension(&value.file_name, settings.format)
                    || !sequence_duration(project, sequence_id).is_some_and(|duration| {
                        image_sequence_range_valid(
                            settings.start_number,
                            duration,
                            output.frame_rate,
                        )
                    })
                {
                    self.value_error("OUTPUT_IMAGE_SEQUENCE", path, value.id.as_str());
                }
            }
            DeliverableKind::CaptionSidecar(settings) => {
                caption::validate(self, value, settings, project, sequence_id, path)
            }
            DeliverableKind::AudioStem(settings) => {
                self.audio_stem(value, settings, project, sequence_id, path)
            }
            DeliverableKind::Scope(settings) => {
                self.scope(value, settings, project, sequence_id, path)
            }
            DeliverableKind::Video(_) => unreachable!("video is validated by the parent module"),
        }
    }

    fn audio_stem(
        &mut self,
        value: &Deliverable,
        settings: &AudioStemOutput,
        project: &Project,
        sequence_id: &SequenceId,
        path: &str,
    ) {
        let expected = match settings.format {
            AudioStemFormat::Wav => "wav",
            AudioStemFormat::Flac => "flac",
        };
        let codec_valid = matches!(
            (settings.format, settings.audio.codec),
            (AudioStemFormat::Wav, AudioCodec::PcmS16Le)
                | (AudioStemFormat::Wav, AudioCodec::PcmS24Le)
                | (AudioStemFormat::Wav, AudioCodec::PcmS32Le)
                | (AudioStemFormat::Flac, AudioCodec::Flac)
        );
        if !extension_is(&value.file_name, expected)
            || !audio_output_valid(&settings.audio)
            || !codec_valid
        {
            self.value_error("OUTPUT_AUDIO_STEM", path, value.id.as_str());
        }
        match &settings.source {
            AudioStemSource::Track { track_id }
                if !sequence(project, sequence_id)
                    .into_iter()
                    .flat_map(|value| &value.tracks)
                    .any(|track| {
                        track.id == *track_id
                            && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
                    }) =>
            {
                self.missing_ref("OUTPUT_STEM_TRACK_NOT_FOUND", track_id.as_str(), path)
            }
            AudioStemSource::Bus { bus_id }
                if !sequence(project, sequence_id)
                        .into_iter()
                        .flat_map(|value| &value.tracks)
                        .any(|track| {
                            matches!(track.kind, TrackKind::Video | TrackKind::Audio)
                                && matches!(&track.routing, TrackRouting::AudioBus { bus_id: value } if value == bus_id)
                        }) =>
            {
                self.value_error("OUTPUT_STEM_BUS", path, value.id.as_str());
            }
            _ => {}
        }
    }

    fn scope(
        &mut self,
        value: &Deliverable,
        settings: &ScopeOutput,
        project: &Project,
        sequence_id: &SequenceId,
        path: &str,
    ) {
        self.time(
            settings.at,
            project.timebase,
            false,
            "OUTPUT_SCOPE_TIME",
            path,
            value.id.as_str(),
        );
        if !ffmpeg_dimensions_valid(settings.width, settings.height)
            || !image_extension(&value.file_name, settings.format)
            || sequence_duration(project, sequence_id).is_none_or(|end| settings.at >= end)
        {
            self.push(
                "OUTPUT_SCOPE",
                Some(value.id.to_string()),
                path,
                "scope settings exceed the default untrusted-plan render budget",
                None,
            );
        }
    }
}

fn sequence<'a>(project: &'a Project, id: &SequenceId) -> Option<&'a Sequence> {
    project.sequences.iter().find(|value| value.id == *id)
}

fn sequence_duration(project: &Project, id: &SequenceId) -> Option<RationalTime> {
    sequence(project, id)?
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter_map(|clip| clip.record_range.end().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

pub(super) fn image_pattern_valid(value: &Deliverable) -> bool {
    matches!(value.kind, DeliverableKind::ImageSequence(_))
        && crate::ImageSequencePattern::parse(&value.file_name).is_some()
        && !value.file_name.contains(['/', '\\', '\0'])
}

fn image_extension(value: &str, format: ImageFormat) -> bool {
    extension_is(
        value,
        match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Exr => "exr",
        },
    )
}

fn extension_is(value: &str, expected: &str) -> bool {
    value
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case(expected))
}
