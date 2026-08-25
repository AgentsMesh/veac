use super::*;
use crate::package::api::*;

fn primitive(name: ApiPrimitive) -> ApiType {
    ApiType::Primitive { name }
}

pub(super) fn callable_semantics(
    effect: ApiSemanticEffect,
    parameter_count: usize,
    has_receiver: bool,
) -> ApiCallableSemantics {
    let dependency = ApiParameterDependency {
        shape_from_shape: false,
        shape_from_leaf: false,
        leaf_from_shape: false,
        leaf_from_leaf: false,
    };
    ApiCallableSemantics {
        effect,
        contains_local_mutation: effect == ApiSemanticEffect::LocalMutation,
        result: ApiResultSemantics {
            shape: ApiStage::Const,
            leaf: ApiStage::Const,
            receiver: has_receiver.then_some(dependency),
            parameters: vec![dependency; parameter_count],
        },
    }
}

fn metadata() -> ApiMetadataV1 {
    ApiMetadataV1::new(
        identity("root", "1.0.0"),
        vec![
            ApiExport::Function {
                name: "make-card".to_owned(),
                parameters: vec![ApiFunctionParameter {
                    name: "title".to_owned(),
                    value_type: primitive(ApiPrimitive::Text),
                    has_default: false,
                }],
                return_type: ApiType::Named {
                    name: ApiTypeName {
                        module: "main.veac".to_owned(),
                        name: "Card".to_owned(),
                    },
                },
                semantics: callable_semantics(ApiSemanticEffect::Pure, 1, false),
            },
            ApiExport::Type {
                name: "Card".to_owned(),
                definition: ApiTypeDefinition::Struct {
                    fields: vec![ApiField {
                        name: "title".to_owned(),
                        value_type: primitive(ApiPrimitive::Text),
                    }],
                },
            },
            ApiExport::Constant {
                name: "default-title".to_owned(),
                value_type: primitive(ApiPrimitive::Text),
            },
        ],
    )
}

#[test]
fn api_metadata_round_trips_canonically_and_has_a_stable_digest() {
    let value = metadata();
    let json = canonical_api_metadata_json(&value).unwrap();
    assert_eq!(parse_api_metadata_json(&json).unwrap(), value);
    assert!(package_api_json_schema().unwrap()["additionalProperties"] == false);
    let empty = ApiMetadataV1::new(identity("root", "1.0.0"), vec![]);
    assert_eq!(
        package_api_digest(&empty).unwrap().as_str(),
        "ee91da46f72db8baa16eeebda1047f51d8092ddbb8e33d576b22c1ec2dee5c3c"
    );
}

#[test]
fn api_metadata_is_closed_sorted_and_recursively_typed() {
    let mut reversed = metadata();
    reversed.exports.reverse();
    assert!(reversed.validate().is_err());

    let unknown = canonical_api_metadata_json(&metadata()).unwrap().replacen(
        r#""effect":"pure""#,
        r#""effect":"pure","ambient":true"#,
        1,
    );
    assert_eq!(
        parse_api_metadata_json(&unknown).unwrap_err().kind(),
        PackageErrorKind::Json
    );

    let complex = ApiMetadataV1::new(
        identity("root", "1.0.0"),
        vec![ApiExport::Function {
            name: "convert".to_owned(),
            parameters: vec![ApiFunctionParameter {
                name: "values".to_owned(),
                value_type: ApiType::Map {
                    key: MapKeyType::Text,
                    value: Box::new(ApiType::List {
                        element: Box::new(ApiType::Tuple {
                            elements: vec![
                                ApiType::Range {
                                    element: ApiPrimitive::Int,
                                },
                                ApiType::Function {
                                    parameters: vec![primitive(ApiPrimitive::Scalar)],
                                    return_type: Box::new(primitive(ApiPrimitive::Bool)),
                                    effect: ApiEffect::Any,
                                },
                            ],
                        }),
                    }),
                },
                has_default: false,
            }],
            return_type: ApiType::Domain {
                name: crate::program::DomainType::PitchPolicy.name().to_owned(),
                opcode: crate::program::DomainType::PitchPolicy.opcode(),
            },
            semantics: callable_semantics(ApiSemanticEffect::GraphEmit, 1, false),
        }],
    )
    .with_domain_capabilities(vec![ApiDomainCapability {
        opcode: crate::program::DomainOperationId::PitchPreserve.opcode(),
        name: crate::program::DomainOperationId::PitchPreserve
            .name()
            .to_owned(),
    }]);
    assert!(complex.validate().is_ok());

    let mut wrong_domain = complex.clone();
    let ApiExport::Function {
        return_type: ApiType::Domain { name, .. },
        ..
    } = &mut wrong_domain.exports[0]
    else {
        unreachable!()
    };
    name.push_str("Wrong");
    assert!(wrong_domain.validate().is_err());

    let mut wrong_capability = complex;
    wrong_capability.domain_capabilities[0]
        .name
        .push_str("_wrong");
    assert!(wrong_capability.validate().is_err());
}

#[test]
fn api_rejects_invalid_names_shapes_and_duplicate_members() {
    let invalid = [
        ApiType::Range {
            element: ApiPrimitive::Time,
        },
        ApiType::Tuple {
            elements: vec![primitive(ApiPrimitive::Int)],
        },
    ];
    for value_type in invalid {
        let value = ApiMetadataV1::new(
            identity("root", "1.0.0"),
            vec![ApiExport::Constant {
                name: "value".to_owned(),
                value_type,
            }],
        );
        assert!(value.validate().is_err());
    }

    let duplicate = ApiMetadataV1::new(
        identity("root", "1.0.0"),
        vec![ApiExport::Type {
            name: "Choice".to_owned(),
            definition: ApiTypeDefinition::Enum {
                variants: vec![
                    ApiEnumVariant {
                        name: "Same".to_owned(),
                        fields: vec![],
                    },
                    ApiEnumVariant {
                        name: "Same".to_owned(),
                        fields: vec![],
                    },
                ],
            },
        }],
    );
    assert!(duplicate.validate().is_err());
}
