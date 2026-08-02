use std::collections::BTreeMap;

use veac_plan::canonical::*;

use super::range;

pub fn apply(id: &str, target: ApplyTarget) -> Apply {
    let suffix = id.strip_prefix("apl_").unwrap_or(id);
    Apply {
        id: ApplyId::new(id).unwrap(),
        enabled: true,
        record_range: range(60, 300),
        target,
        stages: vec![ApplyStage {
            id: ApplyStageId::new(format!("aps_{suffix}")).unwrap(),
            enabled: true,
            active_range: Some(range(30, 120)),
            operation: ApplyOperation::Effect {
                effect: EffectInstance {
                    id: EffectId::new(format!("fx_{suffix}")).unwrap(),
                    effect_type: "video.blur".to_owned(),
                    enabled: true,
                    enable_range: None,
                    parameters: BTreeMap::from([(
                        "radius".to_owned(),
                        ParameterValue::Number { value: 8.0 },
                    )]),
                },
            },
        }],
        mix: ApplyMix::default(),
    }
}

pub fn add_apply_matte(
    envelope: &mut ProjectEnvelope,
    relation_id: &str,
    producer_id: &str,
    apply_id: &str,
    parameters: MatteRelationParameters,
) {
    envelope.project.relations.push(Relation {
        id: RelationId::new(relation_id).unwrap(),
        sequence_id: envelope.project.sequences[0].id.clone(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new(producer_id).unwrap()),
            consumer: RelationEndpoint::apply(ApplyId::new(apply_id).unwrap()),
            parameters,
        },
    });
}
