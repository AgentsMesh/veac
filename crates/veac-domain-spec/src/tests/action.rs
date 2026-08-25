use super::*;

fn action_error(value: DomainOperationContract) -> &'static str {
    rebuild([value], MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_REGISTRY_BYTES)
        .unwrap_err()
        .code()
}

#[test]
fn temporal_lowerings_require_their_exact_primitive_shapes() {
    let point = builtin(DomainOperationId::Point);
    let malformed = operation_with_exposure(
        point.id(),
        point.name(),
        point.exposure().clone(),
        vec![
            operand("x", primitive(PrimitiveType::Scalar), OperandAxis::Leaf),
            operand("y", primitive(PrimitiveType::Scalar), OperandAxis::Leaf),
        ],
        domain(DomainType::Point),
        DomainOperationSemantics::new(
            DomainInstructionKind::DomainConstruct,
            DomainRuntimeAction::Description,
            Effect::Pure,
            Stage::Temporal,
            Some(TemporalLoweringOpcode::ComposePoint),
        ),
    );
    assert_eq!(action_error(malformed), "DOMAIN_OPERATION_ACTION_SHAPE");
}

#[test]
fn graph_actions_require_their_owner_and_child_layouts() {
    let entry = builtin(DomainOperationId::ProjectEntry);
    let malformed_entry = operation_with_exposure(
        entry.id(),
        entry.name(),
        entry.exposure().clone(),
        vec![
            operand(
                "project",
                domain(DomainType::Project),
                OperandAxis::Topology,
            ),
            operand(
                "resource",
                domain(DomainType::Resource),
                OperandAxis::Topology,
            ),
        ],
        domain(DomainType::Project),
        DomainOperationSemantics::new(
            DomainInstructionKind::GraphEmit,
            DomainRuntimeAction::ProjectEntry,
            Effect::GraphEmit,
            Stage::Build,
            None,
        ),
    );
    assert_eq!(
        action_error(malformed_entry),
        "DOMAIN_OPERATION_ACTION_SHAPE"
    );

    let attachment = builtin(DomainOperationId::ProjectWithResource);
    let malformed_attachment = operation_with_exposure(
        attachment.id(),
        attachment.name(),
        attachment.exposure().clone(),
        vec![
            operand(
                "project",
                domain(DomainType::Project),
                OperandAxis::Topology,
            ),
            operand("value", primitive(PrimitiveType::Text), OperandAxis::Leaf),
        ],
        domain(DomainType::Project),
        DomainOperationSemantics::new(
            DomainInstructionKind::GraphEmit,
            DomainRuntimeAction::OwnedAttachment,
            Effect::GraphEmit,
            Stage::Build,
            None,
        ),
    );
    assert_eq!(
        action_error(malformed_attachment),
        "DOMAIN_OPERATION_ACTION_SHAPE"
    );
}

#[test]
fn constructors_and_updates_require_closed_runtime_shapes() {
    let entity = builtin(DomainOperationId::Project);
    let malformed_entity = operation_with_exposure(
        entity.id(),
        entity.name(),
        entity.exposure().clone(),
        vec![operand(
            "settings",
            domain(DomainType::ProjectSettings),
            OperandAxis::Topology,
        )],
        entity.result(),
        DomainOperationSemantics::new(
            DomainInstructionKind::GraphEmit,
            DomainRuntimeAction::EntityConstructor,
            Effect::GraphEmit,
            Stage::Build,
            None,
        ),
    );
    assert_eq!(
        action_error(malformed_entity),
        "DOMAIN_OPERATION_ACTION_SHAPE"
    );

    let relation = builtin(DomainOperationId::RelationTransition);
    let malformed_relation = operation_with_exposure(
        relation.id(),
        relation.name(),
        relation.exposure().clone(),
        relation.operands().to_vec(),
        domain(DomainType::Project),
        DomainOperationSemantics::new(
            DomainInstructionKind::GraphEmit,
            DomainRuntimeAction::RelationConstructor,
            Effect::GraphEmit,
            Stage::Build,
            None,
        ),
    );
    assert_eq!(
        action_error(malformed_relation),
        "DOMAIN_OPERATION_ACTION_SHAPE"
    );

    let update = builtin(DomainOperationId::ItemWithVisual);
    let malformed_update = operation_with_exposure(
        update.id(),
        update.name(),
        update.exposure().clone(),
        vec![
            operand("item", domain(DomainType::Item), OperandAxis::Topology),
            operand("child", domain(DomainType::Resource), OperandAxis::Topology),
        ],
        domain(DomainType::Item),
        DomainOperationSemantics::new(
            DomainInstructionKind::GraphEmit,
            DomainRuntimeAction::NonOwningUpdate,
            Effect::GraphEmit,
            Stage::Build,
            None,
        ),
    );
    assert_eq!(
        action_error(malformed_update),
        "DOMAIN_OPERATION_ACTION_SHAPE"
    );
}
