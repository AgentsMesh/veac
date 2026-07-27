use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

mod keyframe;
mod property;
mod structure;
mod timing;

pub use keyframe::*;
pub use property::*;
pub use structure::*;
pub use timing::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditOperation {
    InsertClip {
        sequence_id: SequenceId,
        track_id: TrackId,
        clip: Box<Clip>,
        before_id: Option<ItemId>,
        after_id: Option<ItemId>,
    },
    RippleInsert {
        sequence_id: SequenceId,
        track_id: TrackId,
        clip: Box<Clip>,
    },
    OverwriteClip {
        sequence_id: SequenceId,
        track_id: TrackId,
        clip: Box<Clip>,
        split_fragments: Vec<OverwriteFragment>,
    },
    RemoveClip {
        clip_id: ItemId,
    },
    RippleDelete {
        clip_id: ItemId,
    },
    MoveClip {
        clip_id: ItemId,
        record_start: RationalTime,
    },
    TrimClip {
        clip_id: ItemId,
        edge: TrimEdge,
        delta: RationalTime,
        ripple: bool,
    },
    SplitClip {
        clip_id: ItemId,
        at: RationalTime,
        right_clip_id: ItemId,
        relation_fragments: Vec<RelationFragmentId>,
    },
    SplitLinked {
        link_relation_id: RelationId,
        offset: RationalTime,
        fragments: Vec<LinkedSplitFragment>,
        right_link_relation_id: RelationId,
    },
    SlipClip {
        clip_id: ItemId,
        source_delta: RationalTime,
    },
    RollEdit {
        left_clip_id: ItemId,
        right_clip_id: ItemId,
        delta: RationalTime,
    },
    SlideClip {
        clip_id: ItemId,
        delta: RationalTime,
    },
    ReplaceSource {
        clip_id: ItemId,
        source: Box<ClipSource>,
        source_mapping: Option<SourceMapping>,
    },
    SetSourceMapping {
        clip_id: ItemId,
        source_mapping: SourceMapping,
    },
    SetMulticamSwitches {
        clip_id: ItemId,
        switches: Vec<MulticamSwitch>,
    },
    SetMulticamGroup {
        group_id: MulticamGroupId,
        sync: MulticamSync,
        angles: Vec<MulticamAngle>,
    },
    InsertAnnotation {
        annotation: Box<Annotation>,
    },
    SetAnnotation {
        annotation: Box<Annotation>,
    },
    RemoveAnnotation {
        annotation_id: AnnotationId,
    },
    SetText {
        clip_id: ItemId,
        text: String,
    },
    SetTemplateState {
        clip_id: ItemId,
        replaceable: Option<SlotConstraint>,
        template_editable_text: bool,
    },
    SetClipEnabled {
        clip_id: ItemId,
        enabled: bool,
    },
    SetVisual {
        clip_id: ItemId,
        visual: Option<VisualProperties>,
    },
    SetAudio {
        clip_id: ItemId,
        audio: Option<AudioProperties>,
    },
    AddEffect {
        clip_id: ItemId,
        effect: EffectInstance,
        before_id: Option<EffectId>,
        after_id: Option<EffectId>,
    },
    RemoveEffect {
        clip_id: ItemId,
        effect_id: EffectId,
    },
    MoveEffect {
        clip_id: ItemId,
        effect_id: EffectId,
        before_id: Option<EffectId>,
        after_id: Option<EffectId>,
    },
    SetTransition {
        clip_id: ItemId,
        transition: Option<Transition>,
    },
    SetVisualProperty {
        clip_id: ItemId,
        property: VisualProperty,
    },
    SetAudioProperty {
        clip_id: ItemId,
        property: AudioProperty,
    },
    SetTextProperty {
        clip_id: ItemId,
        property: TextProperty,
    },
    EditEffectParameter {
        edit: EffectParameterEdit,
    },
    EditKeyframe {
        edit: KeyframeEdit,
    },
    EditStructure {
        edit: StructureEdit,
    },
}
