mod caption;
mod delivery;
mod support;

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
                if !value
                    .target
                    .image_sequence_pattern()
                    .is_some_and(|pattern| image_extension(pattern, settings.format))
                    || !support::sequence_duration(project, sequence_id).is_some_and(|duration| {
                        output.raster.as_ref().is_some_and(|raster| {
                            image_sequence_range_valid(
                                settings.start_number,
                                duration,
                                raster.frame_rate,
                            )
                        })
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
            DeliverableKind::AudioFile(_)
            | DeliverableKind::AnimatedImage(_)
            | DeliverableKind::StillImage(_)
            | DeliverableKind::AdaptivePackage(_) => {
                delivery::validate(self, value, project, output, path)
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
        if !value
            .target
            .file_name()
            .is_some_and(|name| support::extension_is(name, expected))
            || !audio_output_valid(&settings.audio)
            || !codec_valid
        {
            self.value_error("OUTPUT_AUDIO_STEM", path, value.id.as_str());
        }
        match &settings.source {
            AudioMixSource::Track { track_id }
                if !support::source_track_exists(project, sequence_id, track_id) =>
            {
                self.missing_ref("OUTPUT_STEM_TRACK_NOT_FOUND", track_id.as_str(), path)
            }
            AudioMixSource::Bus { bus_id }
                if !support::source_bus_exists(project, sequence_id, bus_id) =>
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
            || !value
                .target
                .file_name()
                .is_some_and(|name| image_extension(name, settings.format))
            || support::sequence_duration(project, sequence_id).is_none_or(|end| settings.at >= end)
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

fn image_extension(value: &str, format: ImageFormat) -> bool {
    support::image_extension(value, format)
}
