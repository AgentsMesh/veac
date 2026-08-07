use super::*;

pub(crate) fn apply(
    id: &str,
    target: ApplyTarget,
    start_ms: i64,
    duration_ms: i64,
    stages: Vec<ApplyStage>,
) -> Apply {
    Apply {
        id: ApplyId::new(id).unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(start_ms), time(duration_ms)).unwrap(),
        target,
        stages,
        mix: ApplyMix::default(),
    }
}

pub(crate) fn effect_stage(id: &str, effect: Effect) -> ApplyStage {
    ApplyStage {
        id: ApplyStageId::new(id).unwrap(),
        enabled: true,
        active_range: None,
        operation: ApplyOperation::Effect {
            effect: video_effect(&format!("fx_{id}"), effect),
        },
    }
}

pub(crate) fn color_stage(id: &str, pipeline: ColorPipeline) -> ApplyStage {
    ApplyStage {
        id: ApplyStageId::new(id).unwrap(),
        enabled: true,
        active_range: None,
        operation: ApplyOperation::Color { pipeline },
    }
}

pub(crate) fn brightness_stage(id: &str, value: f64) -> ApplyStage {
    effect_stage(
        id,
        Effect::VideoColorAdjust {
            brightness: Animatable::constant(value),
            contrast: Animatable::constant(1.0),
            saturation: Animatable::constant(1.0),
        },
    )
}

pub(crate) fn add_apply_matte(
    envelope: &mut ProjectEnvelope,
    sequence: &str,
    producer: &str,
    consumer: &str,
    mode: TrackMatteMode,
    invert: bool,
) {
    envelope.project.relations.push(Relation {
        id: RelationId::new(format!("rel_matte_{consumer}")).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new(producer).unwrap()),
            consumer: RelationEndpoint::apply(ApplyId::new(consumer).unwrap()),
            parameters: MatteRelationParameters { mode, invert },
        },
    });
}
