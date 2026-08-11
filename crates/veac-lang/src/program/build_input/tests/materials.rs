use std::{collections::BTreeMap, sync::Arc};

use crate::program::expression::{PrimitiveType, Value, ValueLookup, ValueType};
use crate::program::{
    FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeRegistry, TypeRegistryBuilder,
};

use super::*;

#[test]
fn every_material_kind_and_authority_builds_the_closed_runtime_struct() {
    let (declarations, types) = fixture();
    for (kind, spelling) in [
        (MaterialInputKind::Video, "video"),
        (MaterialInputKind::Audio, "audio"),
        (MaterialInputKind::Image, "image"),
        (MaterialInputKind::Font, "font"),
        (MaterialInputKind::Lut1d, "lut_1d"),
        (MaterialInputKind::Lut3d, "lut_3d"),
    ] {
        let value = material(kind, MaterialInputAuthority::ProjectMaterial);
        let verified = VerifiedBuildInputs::bind(
            &declarations,
            &types,
            &manifest(vec![binding("asset", value)]),
        )
        .unwrap();
        let Value::Struct(value) =
            ValueLookup::build_value(&verified, &declarations["asset"].id(), "asset").unwrap()
        else {
            panic!("material input must bind as MaterialBinding")
        };
        assert_eq!(value.fields()[0], Value::Text(spelling.into()));
        assert_eq!(value.fields()[3], Value::Text("project_material".into()));
        assert_eq!(value.fields()[4], Value::Text("".into()));
        assert_eq!(value.fields()[5], Value::Integer(2));
        assert_eq!(value.fields()[6], Value::Integer(-1));
    }

    let artifact = material(
        MaterialInputKind::Image,
        MaterialInputAuthority::Artifact {
            artifact_key: "b".repeat(64),
        },
    );
    let verified = VerifiedBuildInputs::bind(
        &declarations,
        &types,
        &manifest(vec![binding("asset", artifact)]),
    )
    .unwrap();
    let Value::Struct(value) =
        ValueLookup::build_value(&verified, &declarations["asset"].id(), "asset").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(value.fields()[3], Value::Text("artifact".into()));
    assert_eq!(value.fields()[4], Value::Text("b".repeat(64).into()));
}

#[test]
fn material_validation_rejects_every_unsafe_path_and_identity_shape() {
    for path in [
        "", "/root", "C:/root", "a\\b", "a//b", "a/./b", "a/../b", "a\nb",
    ] {
        let mut value = material(
            MaterialInputKind::Image,
            MaterialInputAuthority::ProjectMaterial,
        );
        let BuildInputManifestValue::Material { path: actual, .. } = &mut value else {
            unreachable!()
        };
        *actual = path.into();
        assert_eq!(
            value.validate_shape().unwrap_err().code(),
            "PROGRAM_INPUT_MATERIAL"
        );
    }
    for digest in ["a".repeat(63), "A".repeat(64), "g".repeat(64)] {
        let mut value = material(
            MaterialInputKind::Image,
            MaterialInputAuthority::ProjectMaterial,
        );
        let BuildInputManifestValue::Material { sha256, .. } = &mut value else {
            unreachable!()
        };
        *sha256 = digest;
        assert_eq!(
            value.validate_shape().unwrap_err().code(),
            "PROGRAM_INPUT_MATERIAL"
        );
    }
}

fn fixture() -> (BTreeMap<String, BuildInputDeclaration>, TypeRegistry) {
    let fields = [
        ("kind", PrimitiveType::Text),
        ("path", PrimitiveType::Text),
        ("sha256", PrimitiveType::Text),
        ("authority", PrimitiveType::Text),
        ("artifact_key", PrimitiveType::Text),
        ("video_stream", PrimitiveType::Integer),
        ("audio_stream", PrimitiveType::Integer),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (name, kind))| {
        FieldDefinition::new(FieldIndex::new(index as u16), name, ValueType::from(kind))
    })
    .collect::<Vec<_>>();
    let definition = Arc::new(TypeDefinition::new(
        "main.veac",
        "MaterialBinding",
        TypeDefinitionKind::Struct(StructDefinition::new(fields)),
    ));
    let reference = definition.type_ref().clone();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(definition).unwrap();
    builder.bind("MaterialBinding", reference.clone()).unwrap();
    let declaration = BuildInputDeclaration::new(
        "main.veac",
        "asset".into(),
        BuildInputRole::Material,
        ValueType::nominal(reference),
    );
    (
        BTreeMap::from([("asset".into(), declaration)]),
        builder.finish().unwrap(),
    )
}

fn material(kind: MaterialInputKind, authority: MaterialInputAuthority) -> BuildInputManifestValue {
    BuildInputManifestValue::Material {
        kind,
        path: "media/source.mov".into(),
        sha256: "a".repeat(64),
        authority,
        video_stream: Some(2),
        audio_stream: None,
    }
}
