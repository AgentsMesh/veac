use super::api_tests::callable_semantics;
use super::*;
use crate::package::api::*;

#[test]
fn lock_rejects_ambiguous_package_paths_and_identities() {
    let base = locked("alpha", "1.0.0", vec![]);
    let mut root_collision = package_lock(identity("root", "1.0.0"), vec![base.clone()]);
    root_collision.root = identity("alpha", "1.0.0");
    let cases = [
        root_collision,
        package_lock(
            identity("root", "1.0.0"),
            vec![
                base.clone(),
                locked("beta", "1.0.0", vec![dependency("alpha", "1.0.0")]),
            ],
        ),
    ];
    assert!(cases[0].validate().is_err());

    let mut reserved = package_lock(identity("root", "1.0.0"), vec![base.clone()]);
    reserved.packages[0].path = format!("vendor/{PACKAGE_API_FILE}");
    assert!(reserved.validate().is_err());

    let mut nested = cases[1].clone();
    nested.packages[1].path = format!("{}/nested", nested.packages[0].path);
    assert!(nested.validate().is_err());
}

#[test]
fn lock_rejects_root_source_overlap_and_compiler_contract_drift() {
    let base = locked("alpha", "1.0.0", vec![]);
    let mut overlap = package_lock(identity("root", "1.0.0"), vec![base.clone()]);
    overlap.root_files[0].path = format!("{}/nested.veac", base.path);
    overlap.root_content_sha256 = package_content_digest(&overlap.root_files).unwrap();
    assert!(overlap.validate().is_err());

    for axis in ["language", "core", "domain", "schema"] {
        let mut lock = package_lock(identity("root", "1.0.0"), vec![base.clone()]);
        match axis {
            "language" => lock.compatibility.language_version = version("99.0.0"),
            "core" => lock.compatibility.core_version += 1,
            "domain" => lock.compatibility.domain_opset_version += 1,
            "schema" => lock.compatibility.canonical_schema_version += 1,
            _ => unreachable!(),
        }
        assert!(
            lock.validate().is_err(),
            "accepted {axis} compatibility drift"
        );
    }
}

#[test]
fn dependency_and_file_sets_reject_noncanonical_collections() {
    assert!(
        validation::dependencies(&[dependency("same", "1.0.0"), dependency("same", "2.0.0"),])
            .is_err()
    );
    assert!(validation::dependencies(
        &[dependency("beta", "1.0.0"), dependency("alpha", "1.0.0"),]
    )
    .is_err());
    assert!(validation::file_set(&[]).is_err());

    let mut reserved = locked("alpha", "1.0.0", vec![]).files;
    reserved[0].path = format!("nested/{PACKAGE_LOCK_FILE}");
    assert!(validation::file_set(&reserved).is_err());

    let duplicate = LockedFile {
        path: "same.veac".to_owned(),
        sha256: digest('a'),
    };
    assert!(validation::file_set(&[duplicate.clone(), duplicate]).is_err());
}

#[test]
fn scalar_and_digest_contracts_have_fixed_vectors_and_limits() {
    assert!(ExactVersion::new(format!("1.0.0+{}", "a".repeat(129))).is_err());
    assert_eq!(
        Sha256Digest::from_hex(&"b".repeat(64)).unwrap(),
        digest('b')
    );
    assert_eq!(
        sha256_bytes(b"abc").as_str(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let file = LockedFile {
        path: "lib.veac".to_owned(),
        sha256: digest('a'),
    };
    assert_eq!(
        package_content_digest(&[file]).unwrap().as_str(),
        "6df26e14caa0cd5a3bc867582dd8aa8a58a62465dff9a5cbf4b0b27dac955038"
    );
}

#[test]
fn api_preserves_abi_order_and_rejects_recursive_contract_errors() {
    let fields = ["zeta", "alpha"]
        .map(|name| ApiField {
            name: name.to_owned(),
            value_type: primitive(),
        })
        .to_vec();
    let ordered = metadata(ApiExport::Type {
        name: "Layout".to_owned(),
        definition: ApiTypeDefinition::Struct { fields },
    });
    assert!(ordered.validate().is_ok());
    let mut wrong_schema = ordered;
    wrong_schema.schema_version += 1;
    assert!(wrong_schema.validate().is_err());

    assert!(metadata(ApiExport::Type {
        name: "Empty".to_owned(),
        definition: ApiTypeDefinition::Enum { variants: vec![] },
    })
    .validate()
    .is_err());
    let parameters = ["same", "same"]
        .map(|name| ApiFunctionParameter {
            name: name.to_owned(),
            value_type: primitive(),
            has_default: false,
        })
        .to_vec();
    assert!(metadata(ApiExport::Function {
        name: "call".to_owned(),
        parameters,
        return_type: primitive(),
        semantics: callable_semantics(ApiSemanticEffect::LocalMutation, 2, false),
    })
    .validate()
    .is_err());

    let mut nested = primitive();
    for _ in 0..32 {
        nested = ApiType::List {
            element: Box::new(nested),
        };
    }
    assert!(metadata(ApiExport::Constant {
        name: "deep".to_owned(),
        value_type: nested,
    })
    .validate()
    .is_err());
}

#[test]
fn api_callable_receiver_dependencies_match_the_export_kind() {
    let mut method = metadata(ApiExport::Method {
        receiver: ApiTypeName {
            module: "main.veac".to_owned(),
            name: "Card".to_owned(),
        },
        name: "resize".to_owned(),
        parameters: vec![],
        return_type: primitive(),
        semantics: callable_semantics(ApiSemanticEffect::Pure, 0, true),
    });
    assert!(method.validate().is_ok());
    let ApiExport::Method { semantics, .. } = &mut method.exports[0] else {
        unreachable!()
    };
    semantics.result.receiver = None;
    assert!(method.validate().is_err());

    let mut function = metadata(ApiExport::Function {
        name: "make".to_owned(),
        parameters: vec![],
        return_type: primitive(),
        semantics: callable_semantics(ApiSemanticEffect::Pure, 0, false),
    });
    let ApiExport::Function { semantics, .. } = &mut function.exports[0] else {
        unreachable!()
    };
    semantics.result.receiver = Some(ApiParameterDependency {
        shape_from_shape: true,
        shape_from_leaf: false,
        leaf_from_shape: false,
        leaf_from_leaf: true,
    });
    assert!(function.validate().is_err());
}

fn primitive() -> ApiType {
    ApiType::Primitive {
        name: ApiPrimitive::Int,
    }
}

fn metadata(export: ApiExport) -> ApiMetadataV1 {
    ApiMetadataV1::new(identity("root", "1.0.0"), vec![export])
}
