use std::collections::BTreeSet;

use crate::*;

use super::Validator;

impl Validator {
    pub(in crate::validation) fn project_authorship(&mut self, project: &Project) {
        let Some(value) = &project.authorship else {
            return;
        };
        let path = "/project/authorship";
        self.entity_authorship(
            &value.entity,
            &format!("{path}/entity"),
            project.id.as_str(),
        );
        exact(
            value.multicam_groups.iter().map(|entry| &entry.group_id),
            project.multicam_groups.iter().map(|item| &item.id),
            self,
            path,
            project.id.as_str(),
        );
        exact(
            value.annotations.iter().map(|entry| &entry.annotation_id),
            project.annotations.iter().map(|item| &item.id),
            self,
            path,
            project.id.as_str(),
        );
        exact(
            value.deliveries.iter().map(|entry| &entry.render_config_id),
            project.render_configs.iter().map(|item| &item.id),
            self,
            path,
            project.id.as_str(),
        );
        for entity in value
            .multicam_groups
            .iter()
            .map(|entry| &entry.entity)
            .chain(value.annotations.iter().map(|entry| &entry.entity))
            .chain(value.deliveries.iter().map(|entry| &entry.entity))
        {
            self.entity_authorship(entity, path, project.id.as_str());
        }
    }

    pub(in crate::validation) fn sequence_authorship(
        &mut self,
        sequence: &Sequence,
        relations: &[Relation],
        owner_path: &str,
    ) {
        let Some(value) = &sequence.authorship else {
            return;
        };
        let path = format!("{owner_path}/authorship");
        let SequenceAuthorship::Veac {
            entity,
            tracks,
            relations: provenance_relations,
            applies,
        } = value
        else {
            let SequenceAuthorship::Otio { document_sha256 } = value else {
                unreachable!()
            };
            if !document_sha256.is_valid(64)
                || !super::super::values::is_sha256(document_sha256.as_str())
            {
                self.value_error("AUTHORSHIP_OTIO_DIGEST", &path, sequence.id.as_str());
            }
            return;
        };
        self.entity_authorship(entity, &path, sequence.id.as_str());
        exact(
            tracks.iter().map(|entry| &entry.track_id),
            sequence.tracks.iter().map(|item| &item.id),
            self,
            &path,
            sequence.id.as_str(),
        );
        exact(
            provenance_relations.iter().map(|entry| &entry.relation_id),
            relations
                .iter()
                .filter(|item| item.sequence_id == sequence.id)
                .map(|item| &item.id),
            self,
            &path,
            sequence.id.as_str(),
        );
        exact(
            applies.iter().map(|entry| &entry.apply_id),
            sequence.applies.iter().map(|item| &item.id),
            self,
            &path,
            sequence.id.as_str(),
        );
        for authored in tracks
            .iter()
            .map(|entry| &entry.entity)
            .chain(provenance_relations.iter().map(|entry| &entry.entity))
            .chain(applies.iter().map(|entry| &entry.entity))
        {
            self.entity_authorship(authored, &path, sequence.id.as_str());
        }
    }
}

fn exact<'a, T: Ord + 'a>(
    actual: impl Iterator<Item = &'a T>,
    expected: impl Iterator<Item = &'a T>,
    validator: &mut Validator,
    path: &str,
    object_id: &str,
) {
    let actual = actual.collect::<Vec<_>>();
    let sorted_unique = actual.windows(2).all(|pair| pair[0] < pair[1]);
    let actual_count = actual.len();
    let actual = actual.into_iter().collect::<BTreeSet<_>>();
    let expected = expected.collect::<BTreeSet<_>>();
    if !sorted_unique || actual_count != expected.len() || actual != expected {
        validator.value_error("AUTHORSHIP_OWNERSHIP", path, object_id);
    }
}
