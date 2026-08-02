use crate::*;

use super::Validator;

mod components;

impl Validator {
    pub(super) fn clip(
        &mut self,
        clip: &Clip,
        track_kind: TrackKind,
        timebase: u32,
        sample_rate: u32,
        path: &str,
    ) {
        self.metadata(
            &clip.metadata,
            &format!("{path}/metadata"),
            clip.id.as_str(),
        );
        let timed_source = matches!(
            clip.source,
            ClipSource::Media { .. } | ClipSource::Sequence { .. }
        );
        if timed_source != clip.source_mapping.is_some() {
            self.value_error("SOURCE_MAPPING", path, clip.id.as_str());
        }
        self.source_references(clip, timebase, path);
        self.source_media_type(&clip.source, track_kind, path, clip.id.as_str());
        self.track_components(clip, track_kind, path);
        self.source_mapping(clip, timebase, path);
        self.clip_components(clip, track_kind, timebase, sample_rate, path);
    }

    fn source_references(&mut self, clip: &Clip, timebase: u32, path: &str) {
        match &clip.source {
            ClipSource::Media { material_id } | ClipSource::FreezeFrame { material_id, .. } => {
                if !self.material_ids.contains_key(material_id.as_str()) {
                    self.missing_ref(
                        "MATERIAL_NOT_FOUND",
                        material_id.as_str(),
                        &format!("{path}/source/material_id"),
                    );
                }
            }
            ClipSource::Sequence { sequence_id } => {
                if !self.sequence_ids.contains(sequence_id.as_str()) {
                    self.missing_ref(
                        "SEQUENCE_NOT_FOUND",
                        sequence_id.as_str(),
                        &format!("{path}/source/sequence_id"),
                    );
                }
            }
            ClipSource::Multicam { group_id, switches } => self.multicam_clip(
                group_id,
                switches,
                clip.record_range.duration,
                timebase,
                &format!("{path}/source"),
                clip.id.as_str(),
            ),
            ClipSource::Text { text, .. } | ClipSource::Caption { text, .. } if text.is_empty() => {
                self.value_error("EMPTY_TEXT", path, clip.id.as_str());
            }
            _ => {}
        }
        if let ClipSource::FreezeFrame { source_time, .. } = clip.source {
            self.time(
                source_time,
                timebase,
                false,
                "SOURCE_TIME",
                &format!("{path}/source/source_time"),
                clip.id.as_str(),
            );
        }
        if let ClipSource::Caption {
            speaker: Some(speaker),
            ..
        } = &clip.source
        {
            let invalid = speaker.trim().is_empty()
                || speaker.chars().count() > 256
                || speaker
                    .chars()
                    .any(|value| matches!(value, '\0' | '\r' | '\n'));
            if invalid {
                self.value_error("CAPTION_SPEAKER", path, clip.id.as_str());
            }
        }
        let text_style = match &clip.source {
            ClipSource::Text { text, style } => Some((text, style)),
            ClipSource::Caption { text, style, .. } => Some((text, style)),
            _ => None,
        };
        if let Some((text, style)) = text_style {
            for font_id in style.font_refs().filter_map(|font| match font {
                FontRef::Material { material_id } => Some(material_id),
                FontRef::Family { .. } => None,
            }) {
                if self.material_ids.get(font_id.as_str()) == Some(&MaterialKind::Font) {
                    continue;
                }
                self.missing_ref(
                    "FONT_MATERIAL_NOT_FOUND",
                    font_id.as_str(),
                    &format!("{path}/source/style/font"),
                );
            }
            self.text_style(
                text,
                style,
                clip.record_range.duration,
                timebase,
                path,
                clip.id.as_str(),
            );
        }
        if let ClipSource::Generated { generator } = &clip.source {
            self.generator(generator, path, clip.id.as_str());
        }
    }

    fn track_components(&mut self, clip: &Clip, kind: TrackKind, path: &str) {
        if matches!(
            clip.source,
            ClipSource::Text { .. } | ClipSource::Caption { .. }
        ) && clip.visual.is_none()
        {
            self.value_error("TEXT_VISUALS", path, clip.id.as_str());
        }
        if kind == TrackKind::Audio && clip.visual.is_some() {
            self.value_error("TRACK_MEDIA_TYPE", path, clip.id.as_str());
        }
        if matches!(kind, TrackKind::Visual | TrackKind::Caption) && clip.audio.is_some() {
            self.value_error("TRACK_MEDIA_TYPE", path, clip.id.as_str());
        }
        if clip.audio.is_some() && !self.audio_capable(&clip.source) {
            self.value_error("AUDIO_SOURCE_TYPE", path, clip.id.as_str());
        }
    }

    fn audio_capable(&self, source: &ClipSource) -> bool {
        match source {
            ClipSource::Sequence { sequence_id } => self
                .sequence_audio
                .get(sequence_id.as_str())
                .copied()
                .unwrap_or(true),
            ClipSource::Media { .. }
            | ClipSource::Multicam { .. }
            | ClipSource::Generated {
                generator: Generator::Silence,
            } => true,
            _ => false,
        }
    }
}
