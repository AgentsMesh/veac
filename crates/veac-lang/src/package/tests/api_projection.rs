use crate::package::api::{
    from_module_interface, ApiEffect, ApiExport, ApiPrimitive, ApiSemanticEffect, ApiStage,
    ApiType, MapKeyType,
};

use super::identity;

#[path = "api_projection/support.rs"]
mod support;

use support::projection_interface;

#[test]
fn projection_preserves_every_primitive_and_structural_type_variant() {
    let api = from_module_interface(identity("root", "1.0.0"), &projection_interface());
    api.validate().unwrap();
    let primitives = api
        .exports
        .iter()
        .filter_map(|export| match export {
            ApiExport::Constant {
                name,
                value_type: ApiType::Primitive { name: value },
            } if name.starts_with("primitive-") => Some(*value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        primitives,
        vec![
            ApiPrimitive::Int,
            ApiPrimitive::Scalar,
            ApiPrimitive::Time,
            ApiPrimitive::Length,
            ApiPrimitive::Percent,
            ApiPrimitive::Angle,
            ApiPrimitive::Text,
            ApiPrimitive::Color,
            ApiPrimitive::Bool,
            ApiPrimitive::Identifier,
        ]
    );
    let json = crate::package::api::canonical_api_metadata_json(&api).unwrap();
    for token in [
        r#""type":"list""#,
        r#""type":"range""#,
        r#""key":"text""#,
        r#""key":"identifier""#,
        r#""effect":"pure""#,
        r#""effect":"local""#,
        r#""effect":"emit""#,
        r#""effect":"any""#,
        r#""type":"named""#,
        r#""type":"domain""#,
    ] {
        assert!(json.contains(token), "missing projected token {token}");
    }
}

#[test]
fn projection_preserves_callable_effects_stages_and_dependencies() {
    let api = from_module_interface(identity("root", "1.0.0"), &projection_interface());
    let functions = api
        .exports
        .iter()
        .filter_map(|export| match export {
            ApiExport::Function {
                name, semantics, ..
            } => Some((name.as_str(), semantics)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(functions[0].0, "emit");
    assert_eq!(functions[0].1.effect, ApiSemanticEffect::GraphEmit);
    assert_eq!(functions[1].0, "local");
    assert_eq!(functions[1].1.effect, ApiSemanticEffect::LocalMutation);
    assert!(functions[1].1.contains_local_mutation);
    assert_eq!(functions[1].1.result.shape, ApiStage::Temporal);
    assert_eq!(functions[1].1.result.leaf, ApiStage::Temporal);
    assert!(functions[1].1.result.parameters[0].shape_from_shape);
    assert!(functions[1].1.result.parameters[0].leaf_from_shape);
    assert_eq!(functions[2].1.effect, ApiSemanticEffect::Pure);
    assert_eq!(functions[2].1.result.shape, ApiStage::Const);
    assert_eq!(functions[2].1.result.leaf, ApiStage::Build);

    let effects = api.exports.iter().find_map(|export| match export {
        ApiExport::Constant {
            name,
            value_type: ApiType::Tuple { elements },
        } if name == "function-effects" => Some(
            elements
                .iter()
                .map(|element| match element {
                    ApiType::Function { effect, .. } => *effect,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        effects.unwrap(),
        vec![
            ApiEffect::Pure,
            ApiEffect::Local,
            ApiEffect::Emit,
            ApiEffect::Any
        ]
    );
    assert!(api.exports.iter().any(|export| matches!(
        export,
        ApiExport::Constant {
            value_type: ApiType::Map {
                key: MapKeyType::Identifier,
                ..
            },
            ..
        }
    )));
}
