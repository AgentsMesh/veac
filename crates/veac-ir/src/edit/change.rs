use std::collections::BTreeSet;

use crate::*;

pub(super) type ChangeSet = BTreeSet<ChangedObjectId>;

pub(super) trait MarkChanged {
    fn project(&mut self, id: ProjectId);
    fn sequence(&mut self, id: SequenceId);
    fn track(&mut self, id: TrackId);
    fn multicam_group(&mut self, id: MulticamGroupId);
    fn material(&mut self, id: MaterialId);
    fn output(&mut self, id: RenderConfigId);
    fn item(&mut self, id: ItemId);
    fn apply(&mut self, id: ApplyId);
    fn effect(&mut self, id: EffectId);
    fn keyframe(&mut self, id: KeyframeId);
    fn annotation(&mut self, id: AnnotationId);
    fn relation(&mut self, id: RelationId);
}

impl MarkChanged for ChangeSet {
    fn project(&mut self, id: ProjectId) {
        self.insert(ChangedObjectId::Project { id });
    }

    fn sequence(&mut self, id: SequenceId) {
        self.insert(ChangedObjectId::Sequence { id });
    }

    fn track(&mut self, id: TrackId) {
        self.insert(ChangedObjectId::Track { id });
    }

    fn multicam_group(&mut self, id: MulticamGroupId) {
        self.insert(ChangedObjectId::MulticamGroup { id });
    }

    fn material(&mut self, id: MaterialId) {
        self.insert(ChangedObjectId::Material { id });
    }

    fn output(&mut self, id: RenderConfigId) {
        self.insert(ChangedObjectId::Output { id });
    }

    fn item(&mut self, id: ItemId) {
        self.insert(ChangedObjectId::Item { id });
    }

    fn apply(&mut self, id: ApplyId) {
        self.insert(ChangedObjectId::Apply { id });
    }

    fn effect(&mut self, id: EffectId) {
        self.insert(ChangedObjectId::Effect { id });
    }

    fn keyframe(&mut self, id: KeyframeId) {
        self.insert(ChangedObjectId::Keyframe { id });
    }

    fn annotation(&mut self, id: AnnotationId) {
        self.insert(ChangedObjectId::Annotation { id });
    }

    fn relation(&mut self, id: RelationId) {
        self.insert(ChangedObjectId::Relation { id });
    }
}
