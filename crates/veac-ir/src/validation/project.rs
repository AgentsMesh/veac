use std::collections::BTreeSet;

use crate::*;

use super::{values::is_sha256, Validator};

impl Validator {
    pub(super) fn envelope(&mut self, envelope: &ProjectEnvelope) {
        if envelope.schema != SCHEMA_ID {
            self.push(
                "SCHEMA_ID",
                None,
                "/schema",
                format!("expected schema {SCHEMA_ID:?}"),
                None,
            );
        }
        if envelope.schema_version != CURRENT_SCHEMA_VERSION {
            self.push(
                "SCHEMA_VERSION",
                None,
                "/schema_version",
                format!("unsupported schema version {}", envelope.schema_version),
                Some("regenerate the project with a supported schema"),
            );
        }
        if envelope.min_reader_version > CURRENT_SCHEMA_VERSION || envelope.min_reader_version == 0
        {
            self.push(
                "MIN_READER_VERSION",
                None,
                "/min_reader_version",
                "minimum reader version is incompatible",
                None,
            );
        }
        self.project(&envelope.project);
    }

    fn project(&mut self, project: &Project) {
        self.check_id(project.id.is_valid(), project.id.as_str(), "/project/id");
        if project.timebase == 0 {
            self.push(
                "TIMEBASE",
                Some(project.id.to_string()),
                "/project/timebase",
                "project timebase must be greater than zero",
                None,
            );
        }
        if !crate::time::safe_u64(project.revision) {
            self.value_error("REVISION_RANGE", "/project/revision", project.id.as_str());
        }
        self.metadata(&project.metadata, "/project/metadata", project.id.as_str());
        self.render_budget(project);
        self.collect_materials(project);
        self.collect_multicam_groups(project);
        self.collect_sequences(project);
        self.templates(project);

        for output in &project.render_configs {
            self.render_config(output, project);
        }
        for sequence in &project.sequences {
            self.sequence(sequence, project.timebase);
        }
        self.relations(project);
        self.annotations(project);
        self.sequence_cycles(&project.sequences);
        self.applied_operations(project);
    }

    fn collect_materials(&mut self, project: &Project) {
        for material in &project.materials {
            let path = format!("/project/materials/{}", material.id);
            self.check_id(material.id.is_valid(), material.id.as_str(), &path);
            if self
                .material_ids
                .insert(material.id.to_string(), material.kind)
                .is_some()
            {
                self.duplicate("DUPLICATE_MATERIAL_ID", material.id.as_str(), &path);
            }
            self.material(material, &path);
        }
    }

    fn collect_sequences(&mut self, project: &Project) {
        for sequence in &project.sequences {
            let path = format!("/project/sequences/{}", sequence.id);
            self.check_id(sequence.id.is_valid(), sequence.id.as_str(), &path);
            if !self.sequence_ids.insert(sequence.id.to_string()) {
                self.duplicate("DUPLICATE_SEQUENCE_ID", sequence.id.as_str(), &path);
            }
            let audible = sequence.tracks.iter().any(|track| {
                track.state.enabled
                    && !track.state.muted
                    && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
                    && track.clips.iter().any(|clip| {
                        clip.enabled && clip.audio.as_ref().is_some_and(|audio| !audio.muted)
                    })
            });
            self.sequence_audio.insert(sequence.id.to_string(), audible);
        }
        if !self
            .sequence_ids
            .contains(project.entry_sequence_id.as_str())
        {
            self.missing_ref(
                "ENTRY_SEQUENCE_NOT_FOUND",
                project.entry_sequence_id.as_str(),
                "/project/entry_sequence_id",
            );
        }
    }

    fn applied_operations(&mut self, project: &Project) {
        let mut operations = BTreeSet::new();
        for operation in &project.applied_operations {
            let path = "/project/applied_operations";
            self.check_id(operation.id.is_valid(), operation.id.as_str(), path);
            if !is_sha256(&operation.request_hash) {
                self.value_error("OPERATION_HASH", path, operation.id.as_str());
            }
            if !operations.insert(operation.id.as_str()) {
                self.duplicate("DUPLICATE_OPERATION_ID", operation.id.as_str(), path);
            }
        }
    }
}
