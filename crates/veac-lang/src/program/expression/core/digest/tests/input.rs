use crate::program::expression::{
    compile_expression, CoreBuildInputId, CoreInputIdentity, CoreTemporalInputIdentity as Temporal,
    ExpressionContext, PrimitiveType, TypeEnvironment, CORE_VERSION,
};
use veac_ir::{ItemId, MaterialId, SequenceId, TemporalParameterId, TemporalType};

#[test]
fn input_digest_covers_every_temporal_identity_and_value_type() {
    let types = [("input".to_owned(), PrimitiveType::Integer.into())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let mut program = compile_expression("input", &types, &ExpressionContext::empty())
        .unwrap()
        .core()
        .clone();
    let identities = vec![
        Temporal::SequenceTime {
            sequence_id: SequenceId::new("seq_main").unwrap(),
        },
        Temporal::ClipTime {
            item_id: ItemId::new("itm_clip").unwrap(),
        },
        Temporal::SourceTime {
            source_id: MaterialId::new("med_source").unwrap(),
        },
        Temporal::Frame {
            sequence_id: SequenceId::new("seq_main").unwrap(),
        },
        Temporal::Progress {
            item_id: ItemId::new("itm_clip").unwrap(),
        },
    ];
    for identity in identities {
        program.inputs[0].identity = CoreInputIdentity::Temporal(identity);
        assert_ne!(program.input_declarations_digest().as_bytes(), &[0; 32]);
    }
    for value_type in [
        TemporalType::Boolean,
        TemporalType::Integer,
        TemporalType::Scalar,
        TemporalType::Time,
        TemporalType::Length,
        TemporalType::Angle,
        TemporalType::Vec2,
        TemporalType::Point,
        TemporalType::Rect,
        TemporalType::Color,
        TemporalType::Text,
    ] {
        program.inputs[0].identity = CoreInputIdentity::Temporal(Temporal::Parameter {
            parameter_id: TemporalParameterId::new("tpm_value").unwrap(),
            value_type,
        });
        assert_ne!(program.input_declarations_digest().as_bytes(), &[0; 32]);
    }
}

#[test]
fn core_v10_separates_input_identity_and_declaration_digest_domains() {
    let program = compile_expression("1", &TypeEnvironment::new(), &ExpressionContext::empty())
        .unwrap()
        .core()
        .clone();
    assert_eq!(CORE_VERSION, 10);
    assert_eq!(CORE_VERSION, veac_ir::CURRENT_CORE_VERSION);
    assert_eq!(
        program.input_declarations_digest().to_string(),
        "a51b5431440968f92424638514b597f91e8da5b3720007ddf441ab52dd023774"
    );
    assert_eq!(
        CoreBuildInputId::for_symbol("seed").to_string(),
        "95175940bf2247216a91ad47b06cf9223c969081574ba021abc4482f583b9f74"
    );
    assert_eq!(
        CoreBuildInputId::for_declaration("main.veac", "duration", "parameter", "time").to_string(),
        "8aa9e518c9900edda0ac4115cc37ae615dbf5f4792a039b70f73b897ea3634d8"
    );
}
