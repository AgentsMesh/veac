use super::*;

const CLIP: [&str; 4] = ["demo", "main", "visual", "hero"];
const APPLY: [&str; 3] = ["demo", "main", "grade"];

#[test]
fn temporal_effect_and_apply_paths_publish_every_identity_segment() {
    let cases = [
        (
            SourceNodeRef::temporal_clip_effect(
                "main.veac",
                CLIP,
                "blur",
                "amount",
                SourceTemporalProperty::EffectParameter,
            ),
            vec!["demo", "main", "visual", "hero", "blur", "amount"],
        ),
        (
            SourceNodeRef::temporal_apply("main.veac", APPLY, SourceTemporalProperty::ApplyOpacity),
            vec!["demo", "main", "grade"],
        ),
        (
            SourceNodeRef::temporal_apply_mask(
                "main.veac",
                APPLY,
                0,
                SourceTemporalProperty::MaskFeather,
            ),
            vec!["demo", "main", "grade"],
        ),
        (
            SourceNodeRef::temporal_apply_effect(
                "main.veac",
                APPLY,
                ["content", "grade", "amount"],
                SourceTemporalProperty::EffectParameter,
            ),
            vec!["demo", "main", "grade", "content", "grade", "amount"],
        ),
    ];

    for (target, expected) in cases {
        assert_eq!(target.kind(), SourceNodeKind::Temporal);
        assert_eq!(target.path.identifiers(), expected);
        validate_animation_target(target).unwrap();
    }
}

#[test]
fn every_temporal_tail_identifier_is_validated_by_the_contract() {
    let targets = [
        SourceNodeRef::temporal_clip_effect(
            "main.veac",
            CLIP,
            "bad.effect",
            "amount",
            SourceTemporalProperty::EffectParameter,
        ),
        SourceNodeRef::temporal_apply(
            "main.veac",
            ["demo", "main", "bad.apply"],
            SourceTemporalProperty::ApplyOpacity,
        ),
        SourceNodeRef::temporal_apply_effect(
            "main.veac",
            APPLY,
            ["bad.stage", "grade", "amount"],
            SourceTemporalProperty::EffectParameter,
        ),
    ];
    for target in targets {
        assert!(matches!(
            validate_animation_target(target),
            Err(SourceEditError::InvalidNodeId(_))
        ));
    }
}

fn validate_animation_target(target: SourceNodeRef) -> Result<(), SourceEditError> {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::SetBody {
        target,
        site: BodySite::TemporalAnimation {
            property: SourceTemporalProperty::EffectParameter,
        },
        body: BodySource {
            source: "{ progress }".into(),
        },
    }];
    validate_source_edit_contract(&value)
}
