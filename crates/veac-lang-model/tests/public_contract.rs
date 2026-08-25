use std::sync::Arc;

use veac_lang_model::*;

fn primitive(kind: PrimitiveType) -> ValueType {
    ValueType::primitive(kind)
}

fn field(index: u16, name: &str, value: ValueType) -> FieldDefinition {
    FieldDefinition::new(FieldIndex::new(index), name, value)
}

#[test]
fn closed_scalar_surfaces_round_trip_and_expose_ordering() {
    let domains = DomainType::all().collect::<Vec<_>>();
    assert!(!domains.is_empty());
    let mut containers = 0;
    let mut graph_entities = 0;
    let mut topology = 0;
    let mut leaves = 0;
    for domain in domains.iter().copied() {
        assert_eq!(DomainType::from_opcode(domain.opcode()), Some(domain));
        assert_eq!(DomainType::parse(domain.name()), Some(domain));
        assert_eq!(domain.to_string(), domain.name());
        assert_eq!(format!("{domain:?}"), domain.name());
        containers += usize::from(domain.is_container());
        graph_entities += usize::from(domain.is_graph_entity());
        topology += usize::from(domain.requires_topology_axis());
        leaves += usize::from(!domain.requires_topology_axis());
    }
    assert!(containers > 0 && graph_entities >= containers);
    assert_eq!(topology + leaves, domains.len());
    assert!(DomainType::from_opcode(u16::MAX).is_none());
    assert!(DomainType::parse("missing-domain").is_none());

    for primitive_type in PrimitiveType::ALL {
        let token = primitive_type.as_str();
        assert_eq!(PrimitiveType::parse(token), Some(primitive_type));
        assert_eq!(primitive(primitive_type).to_string(), token);
        assert_eq!(
            primitive_type.is_numeric(),
            !matches!(
                primitive_type,
                PrimitiveType::Text
                    | PrimitiveType::Color
                    | PrimitiveType::Boolean
                    | PrimitiveType::Identifier
            )
        );
    }
    assert!(PrimitiveType::parse("unknown").is_none());

    for (effect, token) in FunctionEffect::ALL.into_iter().zip(FunctionEffect::TOKENS) {
        assert_eq!(effect.as_str(), *token);
        assert_eq!(FunctionEffect::parse(token), Some(effect));
        assert_eq!(effect.to_string(), *token);
    }
    assert!(FunctionEffect::parse("unknown").is_none());
    assert_eq!(Stage::Const.join(Stage::Temporal), Stage::Temporal);
    assert_eq!(Stage::Temporal.join(Stage::Build), Stage::Temporal);
    assert_eq!(Stage::Build.join(Stage::Build), Stage::Build);
}

#[test]
fn value_types_cover_every_constructor_and_capability_axis() {
    let text = primitive(PrimitiveType::Text);
    let integer = primitive(PrimitiveType::Integer);
    let domain = ValueType::domain(DomainType::all().next().unwrap());
    let list = ValueType::list(text.clone()).unwrap();
    let range = ValueType::range(integer.clone()).unwrap();
    let map = ValueType::map(text.clone(), list.clone()).unwrap();
    let tuple = ValueType::tuple(vec![integer.clone(), domain.clone()]).unwrap();
    let function = ValueType::function(
        vec![integer.clone(), list.clone()],
        tuple.clone(),
        FunctionEffect::Emit,
    )
    .unwrap();
    assert_eq!(text.as_primitive(), Some(PrimitiveType::Text));
    assert!(domain.as_domain().is_some());
    assert!(function.is_direct_function());
    assert!(list.depth() > text.depth());
    assert!(map.to_string().starts_with("map<text"));
    assert!(range.to_string().starts_with("range<int"));
    assert!(tuple.to_string().starts_with("(int"));
    assert!(function.to_string().contains("effect emit"));

    let nominal_ref = TypeRef::new(TypeId::derive("contract.veac", "Callable"), "Callable");
    let callable = TypeDefinition::new(
        "contract.veac",
        "Callable",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(
            0,
            "run",
            function.clone(),
        )])),
    );
    let domain_holder = TypeDefinition::new(
        "contract.veac",
        "DomainHolder",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Value",
            vec![field(0, "value", domain.clone())],
        )])),
    );
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(callable.clone())).unwrap();
    builder.insert(Arc::new(domain_holder.clone())).unwrap();
    builder
        .bind(
            "contract.Callable",
            nominal_ref.with_diagnostic_name("contract.Callable"),
        )
        .unwrap();
    builder
        .bind(
            "contract.DomainHolder",
            domain_holder
                .type_ref()
                .with_diagnostic_name("contract.DomainHolder"),
        )
        .unwrap();
    let registry = builder.finish().unwrap();
    let callable_type = ValueType::nominal(callable.type_ref().clone());
    let domain_type = ValueType::nominal(domain_holder.type_ref().clone());
    assert_eq!(callable_type.contains_function_in(&registry), Some(true));
    assert_eq!(domain_type.contains_domain_in(&registry), Some(true));
    assert_eq!(callable_type.supports_equality_in(&registry), Some(false));
    assert_eq!(domain_type.is_public_input_in(&registry), Some(false));
    assert_eq!(
        registry.contains_function(callable.type_ref().id()),
        Some(true)
    );
    assert_eq!(
        registry.contains_function(domain_holder.type_ref().id()),
        Some(false)
    );
    assert_eq!(
        registry
            .resolve("contract.Callable")
            .unwrap()
            .diagnostic_name(),
        "contract.Callable"
    );
    assert_eq!(registry.definitions().len(), 2);
    assert_eq!(registry.names().len(), 2);
    assert!(registry
        .definition_handle(callable.type_ref().id())
        .is_some());
}

#[test]
fn nominal_accessors_and_indexes_are_observable() {
    let field = field(0, "title", primitive(PrimitiveType::Text));
    let structure = StructDefinition::new(vec![field.clone()]);
    assert_eq!(structure.fields(), std::slice::from_ref(&field));
    assert!(structure.field("title").is_some());
    assert!(structure.field("missing").is_none());
    let variant = EnumVariantDefinition::new(VariantIndex::new(0), "Ready", vec![]);
    let enumeration = EnumDefinition::new(vec![variant.clone()]);
    assert_eq!(enumeration.variants(), &[variant]);
    assert!(enumeration.variant("Ready").is_some());
    assert!(enumeration.variant("Missing").is_none());
    assert_eq!(FieldIndex::from_position(7).unwrap().index(), 7);
    assert_eq!(VariantIndex::from_position(9).unwrap().value(), 9);
    assert!(FieldIndex::from_position(usize::MAX).is_none());
    let definition =
        TypeDefinition::new("access.veac", "Card", TypeDefinitionKind::Struct(structure));
    assert_eq!(definition.canonical_source_id(), "access.veac");
    assert_eq!(definition.declared_name(), "Card");
    assert!(definition.retained_bytes().unwrap() > 0);
    assert_eq!(definition.type_ref().diagnostic_name(), "Card");
    assert_eq!(
        definition.kind(),
        &TypeDefinitionKind::Struct(StructDefinition::new(vec![field]))
    );
}
