use std::collections::BTreeSet;

use crate::*;

use super::{values::is_sha256, Validator};

mod payload;

impl Validator {
    pub(super) fn annotations(&mut self, project: &Project) {
        let mut ids = BTreeSet::new();
        let mut previous: Option<&str> = None;
        for annotation in &project.annotations {
            let path = format!("/project/annotations/{}", annotation.id);
            self.check_id(annotation.id.is_valid(), annotation.id.as_str(), &path);
            if !ids.insert(annotation.id.as_str()) {
                self.duplicate("DUPLICATE_ANNOTATION_ID", annotation.id.as_str(), &path);
            }
            if previous.is_some_and(|value| value >= annotation.id.as_str()) {
                self.value_error("ANNOTATION_ORDER", &path, annotation.id.as_str());
            }
            previous = Some(annotation.id.as_str());
            self.annotation_target(annotation, &path);
            self.annotation_span(annotation, project, &path);
            if !payload::valid(&annotation.payload, annotation.span) {
                self.value_error("ANNOTATION_PAYLOAD", &path, annotation.id.as_str());
            }
            if annotation
                .provenance
                .as_ref()
                .is_some_and(|value| !provenance_valid(value))
            {
                self.value_error("ANNOTATION_PROVENANCE", &path, annotation.id.as_str());
            }
        }
    }

    fn annotation_target(&mut self, annotation: &Annotation, path: &str) {
        let (valid, exists, id) = match &annotation.target {
            AnnotationTarget::Project => (true, true, annotation.id.as_str()),
            AnnotationTarget::Sequence { sequence_id } => (
                sequence_id.is_valid(),
                self.sequence_ids.contains(sequence_id.as_str()),
                sequence_id.as_str(),
            ),
            AnnotationTarget::Track { track_id } => (
                track_id.is_valid(),
                self.track_ids.contains(track_id.as_str()),
                track_id.as_str(),
            ),
            AnnotationTarget::Clip { clip_id } => (
                clip_id.is_valid(),
                self.item_ids.contains(clip_id.as_str()),
                clip_id.as_str(),
            ),
            AnnotationTarget::Material { material_id } => (
                material_id.is_valid(),
                self.material_ids.contains_key(material_id.as_str()),
                material_id.as_str(),
            ),
            AnnotationTarget::MulticamGroup { group_id } => (
                group_id.is_valid(),
                self.multicam_groups.contains_key(group_id.as_str()),
                group_id.as_str(),
            ),
        };
        self.check_id(valid, id, &format!("{path}/target"));
        if !exists {
            self.missing_ref("ANNOTATION_TARGET_NOT_FOUND", id, &format!("{path}/target"));
        }
    }

    fn annotation_span(&mut self, annotation: &Annotation, project: &Project, path: &str) {
        let intrinsic = matches!(annotation.target, AnnotationTarget::Material { .. });
        match annotation.span {
            AnnotationSpan::Untimed => {}
            AnnotationSpan::Point { at } if intrinsic => self.intrinsic_time(
                at,
                false,
                "ANNOTATION_TIME",
                &format!("{path}/span"),
                annotation.id.as_str(),
            ),
            AnnotationSpan::Point { at } => self.time(
                at,
                project.timebase,
                false,
                "ANNOTATION_TIME",
                &format!("{path}/span"),
                annotation.id.as_str(),
            ),
            AnnotationSpan::Range { range } if intrinsic => {
                self.intrinsic_time(
                    range.start,
                    false,
                    "ANNOTATION_TIME",
                    &format!("{path}/span"),
                    annotation.id.as_str(),
                );
                self.intrinsic_time(
                    range.duration,
                    true,
                    "ANNOTATION_TIME",
                    &format!("{path}/span"),
                    annotation.id.as_str(),
                );
                if range.start.timescale != range.duration.timescale || range.end().is_err() {
                    self.value_error("ANNOTATION_TIME", path, annotation.id.as_str());
                }
            }
            AnnotationSpan::Range { range } => self.time_range(
                range,
                project.timebase,
                "ANNOTATION_TIME",
                &format!("{path}/span"),
                annotation.id.as_str(),
            ),
        }
        if matches!(
            annotation.target,
            AnnotationTarget::Project | AnnotationTarget::MulticamGroup { .. }
        ) && !matches!(annotation.span, AnnotationSpan::Untimed)
        {
            self.value_error("ANNOTATION_TIME_DOMAIN", path, annotation.id.as_str());
        }
        if let AnnotationTarget::Clip { clip_id } = &annotation.target {
            if !clip_span_fits(project, clip_id, annotation.span) {
                self.value_error("ANNOTATION_CLIP_BOUNDS", path, annotation.id.as_str());
            }
        }
    }
}

fn clip_span_fits(project: &Project, id: &ItemId, span: AnnotationSpan) -> bool {
    let Some(clip) = project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id == *id)
    else {
        return true;
    };
    match span {
        AnnotationSpan::Untimed => true,
        AnnotationSpan::Point { at } => at <= clip.record_range.duration,
        AnnotationSpan::Range { range } => range
            .end()
            .is_ok_and(|end| end <= clip.record_range.duration),
    }
}

fn provenance_valid(value: &AnnotationProvenance) -> bool {
    payload::text(&value.producer)
        && is_sha256(&value.request_sha256)
        && is_sha256(&value.response_sha256)
}
