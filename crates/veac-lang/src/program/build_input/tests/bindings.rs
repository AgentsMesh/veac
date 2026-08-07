use crate::program::expression::{CoreBuildInputId, Value, ValueLookup};

use super::*;

#[test]
fn empty_bindings_have_no_digest_and_fail_closed_for_name_lookup() {
    let verified = VerifiedBuildInputs::bind(
        &BTreeMap::new(),
        &crate::program::TypeRegistry::default(),
        &BuildInputManifestV1::empty(),
    )
    .unwrap();
    assert_eq!(verified.digest(), None);
    assert!(verified.residual_bindings().is_empty());
    assert_eq!(ValueLookup::value(&verified, "undeclared"), None);
    assert_eq!(
        ValueLookup::build_value(&verified, &CoreBuildInputId::from_bytes([0; 32]), "missing"),
        None
    );
}

#[test]
fn verified_bindings_resolve_by_stable_declaration_identity() {
    let declarations = declarations(&[
        ("enabled", PrimitiveType::Boolean),
        ("count", PrimitiveType::Integer),
        ("ratio", PrimitiveType::Scalar),
        ("label", PrimitiveType::Text),
        ("duration", PrimitiveType::Time),
        ("width", PrimitiveType::Length),
        ("rotation", PrimitiveType::Angle),
        ("tint", PrimitiveType::Color),
    ]);
    let manifest = manifest(vec![
        binding("enabled", BuildInputManifestValue::Bool { value: true }),
        binding("count", BuildInputManifestValue::Integer { value: 7 }),
        binding(
            "ratio",
            BuildInputManifestValue::Scalar {
                value: "1.25".to_owned(),
            },
        ),
        binding(
            "label",
            BuildInputManifestValue::Text {
                value: "label".to_owned(),
            },
        ),
        binding(
            "duration",
            BuildInputManifestValue::Time {
                value: "1250ms".to_owned(),
            },
        ),
        binding(
            "width",
            BuildInputManifestValue::Length {
                value: "12.5px".to_owned(),
            },
        ),
        binding(
            "rotation",
            BuildInputManifestValue::Angle {
                value: "-45deg".to_owned(),
            },
        ),
        binding(
            "tint",
            BuildInputManifestValue::Color {
                value: "#ABCDEF80".to_owned(),
            },
        ),
    ]);
    let verified = VerifiedBuildInputs::bind(
        &declarations,
        &crate::program::TypeRegistry::default(),
        &manifest,
    )
    .unwrap();
    assert_eq!(verified.digest().map(str::len), Some(64));
    let tint = &declarations["tint"];
    assert_eq!(
        ValueLookup::build_value(&verified, &tint.id(), tint.name()),
        Some(&Value::Color("#abcdef80".into()))
    );
}

#[test]
fn binding_rejects_invalid_identity_unknown_duplicate_type_and_missing_values() {
    let declarations = declarations(&[("count", PrimitiveType::Integer)]);

    let mut invalid_identity = manifest(vec![binding(
        "count",
        BuildInputManifestValue::Integer { value: 1 },
    )]);
    invalid_identity.schema_version += 1;
    assert_error(
        &declarations,
        &invalid_identity,
        "PROGRAM_INPUT_MANIFEST_VERSION",
    );

    let unknown = manifest(vec![binding(
        "other",
        BuildInputManifestValue::Integer { value: 1 },
    )]);
    assert_error(&declarations, &unknown, "PROGRAM_INPUT_UNKNOWN");

    let duplicate = manifest(vec![
        binding("count", BuildInputManifestValue::Integer { value: 1 }),
        binding("count", BuildInputManifestValue::Integer { value: 2 }),
    ]);
    assert_error(&declarations, &duplicate, "PROGRAM_INPUT_DUPLICATE");

    let mismatch = manifest(vec![binding(
        "count",
        BuildInputManifestValue::Bool { value: true },
    )]);
    assert_error(&declarations, &mismatch, "PROGRAM_INPUT_TYPE_MISMATCH");
    assert_error(
        &declarations,
        &BuildInputManifestV1::empty(),
        "PROGRAM_INPUT_MISSING",
    );
}

fn assert_error(
    declarations: &BTreeMap<String, BuildInputDeclaration>,
    manifest: &BuildInputManifestV1,
    code: &str,
) {
    let error = match VerifiedBuildInputs::bind(
        declarations,
        &crate::program::TypeRegistry::default(),
        manifest,
    ) {
        Ok(_) => panic!("binding unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(error.code(), code);
}
