use super::support::*;
use crate::program::expression::{
    evaluate_compiled, CoreBuildInputId, CoreTemporalInputIdentity, Environment, Stage,
};
use veac_ir::{ItemId, MaterialId, SequenceId, TemporalParameterId, TemporalType};

fn digest(identity: CoreTemporalInputIdentity) -> [u8; 32] {
    *compile("input", &[("input", identity)])
        .core()
        .input_declarations_digest()
        .as_bytes()
}

#[test]
fn build_identity_is_stable_and_symbol_scoped() {
    assert_eq!(
        CoreBuildInputId::for_symbol("gain"),
        CoreBuildInputId::for_symbol("gain")
    );
    assert_ne!(
        CoreBuildInputId::for_symbol("gain"),
        CoreBuildInputId::for_symbol("opacity")
    );
}

#[test]
fn temporal_clock_owner_kinds_are_closed_and_non_aliasing() {
    let identities = [
        CoreTemporalInputIdentity::SequenceTime {
            sequence_id: SequenceId::new("seq_main").unwrap(),
        },
        CoreTemporalInputIdentity::ClipTime {
            item_id: ItemId::new("itm_main").unwrap(),
        },
        CoreTemporalInputIdentity::SourceTime {
            source_id: MaterialId::new("med_main").unwrap(),
        },
        CoreTemporalInputIdentity::Frame {
            sequence_id: SequenceId::new("seq_main").unwrap(),
        },
        CoreTemporalInputIdentity::Progress {
            item_id: ItemId::new("itm_main").unwrap(),
        },
    ];
    let digests = identities.into_iter().map(digest).collect::<Vec<_>>();
    for (index, value) in digests.iter().enumerate() {
        assert!(!digests[..index].contains(value));
    }
}

#[test]
fn input_declaration_digest_uses_ordinal_identity_and_type_only() {
    let compiled = compile(
        "(first, second, 1)",
        &[
            (
                "first",
                CoreTemporalInputIdentity::Parameter {
                    parameter_id: TemporalParameterId::new("tpm_first").unwrap(),
                    value_type: TemporalType::Scalar,
                },
            ),
            (
                "second",
                CoreTemporalInputIdentity::Parameter {
                    parameter_id: TemporalParameterId::new("tpm_second").unwrap(),
                    value_type: TemporalType::Scalar,
                },
            ),
        ],
    );
    let original = compiled.core().input_declarations_digest();

    let mut presentation = compiled.core().clone();
    presentation.inputs[0].name = "renamed".into();
    presentation.inputs[0].span = 500..900;
    assert_eq!(presentation.input_declarations_digest(), original);

    let mut ordinal = compiled.core().clone();
    ordinal.inputs.swap(0, 1);
    assert_ne!(ordinal.input_declarations_digest(), original);

    let mut identity = compiled.core().clone();
    identity.inputs[0].identity = identity.inputs[1].identity.clone();
    assert_ne!(identity.input_declarations_digest(), original);

    let integer_type = compiled.core().blocks()[0]
        .instructions()
        .iter()
        .find(|value| {
            compiled
                .core()
                .value_type(value.type_id())
                .unwrap()
                .to_string()
                == "int"
        })
        .unwrap()
        .type_id();
    let mut changed_type = compiled.core().clone();
    changed_type.inputs[0].type_id = integer_type;
    assert_ne!(changed_type.input_declarations_digest(), original);
}

#[test]
fn temporal_input_metadata_is_fixed_shape_and_temporal_leaf() {
    let compiled = compile(
        "input",
        &[("input", parameter("metadata", TemporalType::Scalar))],
    );
    let input = &compiled.core().blocks()[0].instructions()[0];
    assert_eq!(input.metadata().shape_stage(), Stage::Const);
    assert_eq!(input.metadata().leaf_stage(), Stage::Temporal);
}

#[test]
fn concrete_runtime_requires_residualization_for_temporal_input() {
    let compiled = compile(
        "input",
        &[("input", parameter("runtime", TemporalType::Scalar))],
    );
    let error = evaluate_compiled(&compiled, &Environment::new()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TEMPORAL_INPUT_REQUIRED");
}
