use crate::program::expression::{Effect, FunctionEffect, MapKeyType, PrimitiveType, Stage};
use crate::program::{
    CompilerDatabase, ModuleCallableSemantics, ModuleConstantInterface, ModuleFunctionInterface,
    ModuleInterface, ModuleInterfaceType, ModuleParameterDependency, ModuleParameterInterface,
    ModuleResultSemantics, ModuleTypeName,
};

fn primitive(value: PrimitiveType) -> ModuleInterfaceType {
    ModuleInterfaceType::Primitive(value)
}

fn function_type(effect: FunctionEffect) -> ModuleInterfaceType {
    ModuleInterfaceType::Function {
        parameters: vec![primitive(PrimitiveType::Time)],
        return_type: Box::new(primitive(PrimitiveType::Boolean)),
        effect,
    }
}

fn constant(name: &str, value_type: ModuleInterfaceType) -> ModuleConstantInterface {
    ModuleConstantInterface {
        name: name.to_owned(),
        value_type,
    }
}

fn semantics(effect: Effect, shape: Stage, leaf: Stage) -> ModuleCallableSemantics {
    ModuleCallableSemantics {
        effect,
        contains_local_mutation: effect == Effect::LocalMutation,
        result: ModuleResultSemantics {
            shape,
            leaf,
            receiver: None,
            parameters: vec![ModuleParameterDependency {
                shape_from_shape: true,
                shape_from_leaf: false,
                leaf_from_shape: true,
                leaf_from_leaf: false,
            }],
        },
    }
}

fn structural_types() -> [ModuleConstantInterface; 7] {
    [
        constant(
            "list-value",
            ModuleInterfaceType::List(Box::new(primitive(PrimitiveType::Text))),
        ),
        constant(
            "range-value",
            ModuleInterfaceType::Range(PrimitiveType::Integer),
        ),
        constant(
            "text-map",
            ModuleInterfaceType::Map {
                key: MapKeyType::Text,
                value: Box::new(primitive(PrimitiveType::Color)),
            },
        ),
        constant(
            "identifier-map",
            ModuleInterfaceType::Map {
                key: MapKeyType::Identifier,
                value: Box::new(primitive(PrimitiveType::Length)),
            },
        ),
        constant(
            "function-effects",
            ModuleInterfaceType::Tuple(
                FunctionEffect::ALL.into_iter().map(function_type).collect(),
            ),
        ),
        constant(
            "named-value",
            ModuleInterfaceType::Named(ModuleTypeName {
                source_id: "types.veac".to_owned(),
                name: "Card".to_owned(),
            }),
        ),
        constant(
            "domain-value",
            ModuleInterfaceType::Domain {
                name: crate::program::DomainType::PitchPolicy.name().to_owned(),
                opcode: crate::program::DomainType::PitchPolicy.opcode(),
            },
        ),
    ]
}

fn callable(name: &str, effect: Effect, shape: Stage, leaf: Stage) -> ModuleFunctionInterface {
    ModuleFunctionInterface {
        name: name.to_owned(),
        parameters: vec![ModuleParameterInterface {
            name: "input".to_owned(),
            value_type: primitive(PrimitiveType::Time),
            has_default: true,
        }],
        return_type: primitive(PrimitiveType::Time),
        semantics: semantics(effect, shape, leaf),
    }
}

pub(super) fn projection_interface() -> ModuleInterface {
    let mut constants = PrimitiveType::ALL
        .into_iter()
        .enumerate()
        .map(|(index, value)| constant(&format!("primitive-{index}"), primitive(value)))
        .collect::<Vec<_>>();
    constants.extend(structural_types());
    ModuleInterface {
        source_id: "main.veac".to_owned(),
        revision: CompilerDatabase::default().source_revision("main.veac", "module {}"),
        functions: vec![
            callable("pure", Effect::Pure, Stage::Const, Stage::Build),
            callable(
                "local",
                Effect::LocalMutation,
                Stage::Temporal,
                Stage::Temporal,
            ),
            callable("emit", Effect::GraphEmit, Stage::Build, Stage::Const),
        ],
        methods: vec![],
        types: vec![],
        constants,
        domain_capabilities: vec![],
    }
}
