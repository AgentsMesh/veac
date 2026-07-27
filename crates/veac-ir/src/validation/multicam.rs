use std::collections::BTreeSet;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn collect_multicam_groups(&mut self, project: &Project) {
        let mut previous: Option<&str> = None;
        for group in &project.multicam_groups {
            let path = format!("/project/multicam_groups/{}", group.id);
            self.check_id(group.id.is_valid(), group.id.as_str(), &path);
            if previous.is_some_and(|id| id >= group.id.as_str()) {
                self.value_error("MULTICAM_GROUP_ORDER", &path, group.id.as_str());
            }
            previous = Some(group.id.as_str());
            let angles = self.multicam_group(project, group, &path);
            if self
                .multicam_groups
                .insert(group.id.to_string(), angles)
                .is_some()
            {
                self.duplicate("DUPLICATE_MULTICAM_GROUP_ID", group.id.as_str(), &path);
            }
        }
    }

    fn multicam_group(
        &mut self,
        project: &Project,
        group: &MulticamGroup,
        path: &str,
    ) -> BTreeSet<String> {
        let mut angles = BTreeSet::new();
        let mut previous: Option<&str> = None;
        if group.angles.len() < 2 {
            self.value_error("MULTICAM_ANGLE_COUNT", path, group.id.as_str());
        }
        for angle in &group.angles {
            let angle_path = format!("{path}/angles/{}", angle.id);
            self.check_id(angle.id.is_valid(), angle.id.as_str(), &angle_path);
            if previous.is_some_and(|id| id >= angle.id.as_str()) {
                self.value_error("MULTICAM_ANGLE_ORDER", &angle_path, angle.id.as_str());
            }
            previous = Some(angle.id.as_str());
            if !angles.insert(angle.id.to_string()) {
                self.duplicate(
                    "DUPLICATE_MULTICAM_ANGLE_ID",
                    angle.id.as_str(),
                    &angle_path,
                );
            }
            self.multicam_angle(project, group, angle, &angle_path);
        }
        if !angles.contains(group.sync.reference_angle_id.as_str()) {
            self.missing_ref(
                "MULTICAM_REFERENCE_ANGLE_NOT_FOUND",
                group.sync.reference_angle_id.as_str(),
                &format!("{path}/sync/reference_angle_id"),
            );
        }
        angles
    }

    fn multicam_angle(
        &mut self,
        project: &Project,
        group: &MulticamGroup,
        angle: &MulticamAngle,
        path: &str,
    ) {
        self.time(
            angle.source_offset,
            project.timebase,
            false,
            "MULTICAM_SOURCE_OFFSET",
            &format!("{path}/source_offset"),
            angle.id.as_str(),
        );
        let material = project
            .materials
            .iter()
            .find(|material| material.id == angle.material_id);
        if material.is_none_or(|material| material.kind != MaterialKind::Video) {
            self.missing_ref(
                "MULTICAM_VIDEO_MATERIAL_NOT_FOUND",
                angle.material_id.as_str(),
                &format!("{path}/material_id"),
            );
        } else if group.sync.basis == MulticamSyncBasis::Audio
            && material.is_some_and(|material| {
                material
                    .probe
                    .as_ref()
                    .is_some_and(|probe| probe.selected_audio_stream.is_none())
            })
        {
            self.value_error("MULTICAM_SYNC_AUDIO_MISSING", path, angle.id.as_str());
        }
    }

    pub(super) fn multicam_clip(
        &mut self,
        group_id: &MulticamGroupId,
        switches: &[MulticamSwitch],
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        let Some(angles) = self.multicam_groups.get(group_id.as_str()).cloned() else {
            self.missing_ref("MULTICAM_GROUP_NOT_FOUND", group_id.as_str(), path);
            return;
        };
        let mut expected = RationalTime {
            value: 0,
            timescale: timebase,
        };
        for value in switches {
            self.time_range(
                value.range,
                timebase,
                "MULTICAM_SWITCH_RANGE",
                path,
                item_id,
            );
            if value.range.start != expected {
                self.value_error("MULTICAM_SWITCH_PARTITION", path, item_id);
            }
            if !angles.contains(value.angle_id.as_str()) {
                self.missing_ref(
                    "MULTICAM_SWITCH_ANGLE_NOT_FOUND",
                    value.angle_id.as_str(),
                    path,
                );
            }
            if let Ok(end) = value.range.end() {
                expected = end;
            }
        }
        if switches.is_empty() || expected != duration {
            self.value_error("MULTICAM_SWITCH_PARTITION", path, item_id);
        }
    }
}
