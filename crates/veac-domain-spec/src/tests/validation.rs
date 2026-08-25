use super::*;

fn error(value: DomainOperationContract) -> &'static str {
    rebuild(
        std::iter::once(value),
        MAX_DOMAIN_OPERATIONS,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err()
    .code()
}

#[test]
fn stale_name_and_operand_layouts_are_rejected() {
    let bad_name = operation(
        DomainOperationId::Canvas,
        "bad.name",
        Vec::new(),
        domain(DomainType::Canvas),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(bad_name), "DOMAIN_OPERATION_NAME");

    let duplicate = operation(
        DomainOperationId::Canvas,
        "canvas",
        vec![
            operand(
                "width",
                primitive(PrimitiveType::Length),
                OperandAxis::Topology,
            ),
            operand(
                "width",
                primitive(PrimitiveType::Length),
                OperandAxis::Topology,
            ),
        ],
        domain(DomainType::Canvas),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(duplicate), "DOMAIN_OPERATION_OPERAND");

    let changed = operation(
        DomainOperationId::Canvas,
        "canvas",
        vec![
            operand(
                "width",
                primitive(PrimitiveType::Length),
                OperandAxis::Topology,
            ),
            operand(
                "depth",
                primitive(PrimitiveType::Length),
                OperandAxis::Topology,
            ),
        ],
        domain(DomainType::Canvas),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(changed), "DOMAIN_OPERATION_CONTRACT");
}

#[test]
fn operand_count_and_topology_axes_are_verified() {
    let operands = (0..=MAX_DOMAIN_OPERATION_OPERANDS)
        .map(|index| {
            operand(
                &format!("value_{index}"),
                primitive(PrimitiveType::Integer),
                OperandAxis::Topology,
            )
        })
        .collect();
    let too_many = operation(
        DomainOperationId::Canvas,
        "canvas",
        operands,
        domain(DomainType::Canvas),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(too_many), "DOMAIN_OPERATION_OPERAND_LIMIT");

    let topology_as_leaf = operation(
        DomainOperationId::Canvas,
        "canvas",
        vec![operand(
            "project",
            domain(DomainType::Project),
            OperandAxis::Leaf,
        )],
        domain(DomainType::Canvas),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(topology_as_leaf), "DOMAIN_OPERATION_AXIS");
}

#[test]
fn instruction_effect_and_result_contracts_are_verified() {
    let effect = operation(
        DomainOperationId::Canvas,
        "canvas",
        Vec::new(),
        domain(DomainType::Canvas),
        DomainInstructionKind::DomainConstruct,
        Effect::LocalMutation,
    );
    assert_eq!(error(effect.clone()), "DOMAIN_OPERATION_EFFECT");
    let digest = super::super::digest::registry(
        DomainOpsetVersion::CURRENT,
        std::slice::from_ref(&effect).iter(),
    );
    assert_ne!(digest, DomainOperationRegistry::standard().digest());

    let primitive_result = operation(
        DomainOperationId::Canvas,
        "canvas",
        Vec::new(),
        primitive(PrimitiveType::Integer),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(primitive_result), "DOMAIN_OPERATION_RESULT");

    let container_construct = operation(
        DomainOperationId::Canvas,
        "canvas",
        Vec::new(),
        domain(DomainType::Project),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    assert_eq!(error(container_construct), "DOMAIN_OPERATION_RESULT");

    let emit_without_topology = operation(
        DomainOperationId::Project,
        "project",
        vec![operand(
            "value",
            primitive(PrimitiveType::Text),
            OperandAxis::Leaf,
        )],
        domain(DomainType::Project),
        DomainInstructionKind::GraphEmit,
        Effect::GraphEmit,
    );
    assert_eq!(error(emit_without_topology), "DOMAIN_OPERATION_RESULT");
}

#[test]
fn digest_tags_every_primitive_and_the_opset_version() {
    let primitives = [
        PrimitiveType::Integer,
        PrimitiveType::Scalar,
        PrimitiveType::Time,
        PrimitiveType::Length,
        PrimitiveType::Percent,
        PrimitiveType::Angle,
        PrimitiveType::Text,
        PrimitiveType::Color,
        PrimitiveType::Boolean,
        PrimitiveType::Identifier,
    ];
    let digests = primitives
        .into_iter()
        .map(|value| {
            let contract = operation(
                DomainOperationId::GeneratorSolid,
                "generator_solid",
                vec![operand("value", primitive(value), OperandAxis::Leaf)],
                domain(DomainType::Generator),
                DomainInstructionKind::DomainConstruct,
                Effect::Pure,
            );
            super::super::digest::registry(
                DomainOpsetVersion::CURRENT,
                std::slice::from_ref(&contract).iter(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(digests.len(), primitives.len());
    let contracts = all_contracts();
    let other_version = super::super::digest::registry(DomainOpsetVersion::V1, contracts.iter());
    assert_ne!(other_version, DomainOperationRegistry::standard().digest());
}
