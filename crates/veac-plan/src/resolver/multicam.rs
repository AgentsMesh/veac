use veac_ir::{Clip, MulticamGroupId, MulticamSwitch, MulticamSyncBasis, RationalTime, TimeRange};

use super::{bounds::BoundContext, material::InputUsage, PlanResolver};
use crate::{ResolvedClipSource, ResolvedMulticamAngle, ResolvedMulticamSource};

impl PlanResolver<'_> {
    pub(super) fn resolve_multicam(
        &mut self,
        clip: &Clip,
        group_id: &MulticamGroupId,
        switches: &[MulticamSwitch],
        visual: bool,
        audio: bool,
        path: &str,
    ) -> Option<ResolvedClipSource> {
        let group = self
            .envelope
            .project
            .multicam_groups
            .iter()
            .find(|group| group.id == *group_id)
            .cloned()?;
        let sync_audio = group.sync.basis == MulticamSyncBasis::Audio;
        let usage = InputUsage {
            video: visual,
            audio: audio || sync_audio,
            font: false,
        };
        let mut angles = Vec::with_capacity(group.angles.len());
        for angle in &group.angles {
            let input = self.material_input(&angle.material_id, usage)?;
            self.check_multicam_bounds(&input, clip, angle, switches, usage, path);
            angles.push(ResolvedMulticamAngle {
                id: angle.id.clone(),
                input_id: input.id,
                video_stream: input.video.as_ref()?.selection,
                audio_stream: (audio || sync_audio)
                    .then(|| input.audio.as_ref().map(|value| value.selection))
                    .flatten(),
                source_offset: angle.source_offset,
            });
        }
        Some(ResolvedClipSource::Multicam {
            source: ResolvedMulticamSource {
                group_id: group.id,
                sync: group.sync,
                angles,
                switches: switches.to_vec(),
            },
        })
    }

    fn check_multicam_bounds(
        &mut self,
        input: &crate::ResolvedInput,
        clip: &Clip,
        angle: &veac_ir::MulticamAngle,
        switches: &[MulticamSwitch],
        usage: InputUsage,
        path: &str,
    ) {
        let used: Vec<_> = switches
            .iter()
            .filter(|value| value.angle_id == angle.id)
            .collect();
        let (Some(first), Some(last)) = (used.first(), used.last()) else {
            return;
        };
        let Ok(start) = angle.source_offset.checked_add(first.range.start) else {
            self.push_internal(
                "MULTICAM_TIME_ARITHMETIC",
                clip.id.to_string(),
                "multicam source start overflowed".to_owned(),
            );
            return;
        };
        let Some(end) = last
            .range
            .end()
            .ok()
            .and_then(|value| angle.source_offset.checked_add(value).ok())
        else {
            self.push_internal(
                "MULTICAM_TIME_ARITHMETIC",
                clip.id.to_string(),
                "multicam source end overflowed".to_owned(),
            );
            return;
        };
        let duration = RationalTime::new(end.value - start.value, end.timescale).unwrap();
        self.check_source_bounds(
            input,
            TimeRange { start, duration },
            usage,
            BoundContext::new(clip, path, veac_ir::SourceOutOfRangePolicy::Strict),
            false,
        );
    }
}
