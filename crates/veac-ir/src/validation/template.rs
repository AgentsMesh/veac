use std::collections::BTreeMap;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn templates(&mut self, project: &Project) {
        let uses = material_uses(project);
        for sequence in &project.sequences {
            for track in &sequence.tracks {
                for clip in &track.clips {
                    let path = format!(
                        "/project/sequences/{}/tracks/{}/clips/{}",
                        sequence.id, track.id, clip.id
                    );
                    self.template_clip(clip, track.kind, project.timebase, &uses, &path);
                }
            }
        }
    }

    fn template_clip(
        &mut self,
        clip: &Clip,
        track_kind: TrackKind,
        timebase: u32,
        uses: &BTreeMap<String, usize>,
        path: &str,
    ) {
        let Some(slot) = &clip.replaceable else {
            if clip.template_editable_text {
                self.value_error("TEMPLATE_EDITABLE_TEXT", path, clip.id.as_str());
            }
            return;
        };
        let slot_path = format!("{path}/replaceable");
        if clip.template_editable_text
            && (slot.kind != SlotKind::Text || !matches!(clip.source, ClipSource::Text { .. }))
        {
            self.value_error("TEMPLATE_EDITABLE_TEXT", path, clip.id.as_str());
        }
        if slot.label.trim().is_empty() || slot.label.len() > SLOT_LABEL_MAX_BYTES {
            self.value_error("TEMPLATE_SLOT_LABEL", &slot_path, clip.id.as_str());
        }
        if let Some(duration) = slot.min_source_duration {
            if slot.kind != SlotKind::Video {
                self.value_error("TEMPLATE_SLOT_MIN_KIND", &slot_path, clip.id.as_str());
            }
            self.time(
                duration,
                timebase,
                true,
                "TEMPLATE_SLOT_MIN_DURATION",
                &format!("{slot_path}/min_source_duration"),
                clip.id.as_str(),
            );
        }
        if slot.kind == SlotKind::Text {
            self.text_template_slot(clip, track_kind, slot, &slot_path);
            return;
        }
        self.media_template_slot(clip, track_kind, slot, uses, &slot_path);
    }

    fn text_template_slot(
        &mut self,
        clip: &Clip,
        track_kind: TrackKind,
        slot: &SlotConstraint,
        path: &str,
    ) {
        if track_kind != TrackKind::Visual {
            self.value_error("TEMPLATE_SLOT_TRACK", path, clip.id.as_str());
        }
        if slot.fill != FillMode::FitDuration {
            self.value_error("TEMPLATE_SLOT_TEXT_FILL", path, clip.id.as_str());
        }
        if !matches!(clip.source, ClipSource::Text { .. }) {
            self.value_error("TEMPLATE_SLOT_SOURCE", path, clip.id.as_str());
        }
    }

    fn media_template_slot(
        &mut self,
        clip: &Clip,
        track_kind: TrackKind,
        slot: &SlotConstraint,
        uses: &BTreeMap<String, usize>,
        path: &str,
    ) {
        if clip.template_editable_text {
            self.value_error("TEMPLATE_SLOT_TEXT_POLICY", path, clip.id.as_str());
        }
        if !matches!(track_kind, TrackKind::Video | TrackKind::Visual) {
            self.value_error("TEMPLATE_SLOT_TRACK", path, clip.id.as_str());
        }
        if clip.visual.is_none() {
            self.value_error("TEMPLATE_SLOT_VISUAL", path, clip.id.as_str());
        }
        let ClipSource::Media { material_id } = &clip.source else {
            self.value_error("TEMPLATE_SLOT_SOURCE", path, clip.id.as_str());
            return;
        };
        if self
            .material_ids
            .get(material_id.as_str())
            .is_some_and(|kind| !slot.kind.accepts(*kind))
        {
            self.value_error("TEMPLATE_SLOT_KIND", path, clip.id.as_str());
        }
        if uses.get(material_id.as_str()).copied() != Some(1) {
            self.value_error("TEMPLATE_SLOT_OWNERSHIP", path, clip.id.as_str());
        }
    }
}

fn material_uses(project: &Project) -> BTreeMap<String, usize> {
    let mut uses = BTreeMap::new();
    for angle in project
        .multicam_groups
        .iter()
        .flat_map(|group| &group.angles)
    {
        increment(&mut uses, &angle.material_id);
    }
    for clip in project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        match &clip.source {
            ClipSource::Media { material_id } | ClipSource::FreezeFrame { material_id, .. } => {
                increment(&mut uses, material_id)
            }
            _ => {}
        }
    }
    uses
}

fn increment(uses: &mut BTreeMap<String, usize>, id: &MaterialId) {
    *uses.entry(id.to_string()).or_default() += 1;
}
