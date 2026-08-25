use super::*;

#[test]
fn transform_v6_is_an_atomic_typed_composition() {
    let registry = DomainOperationRegistry::standard();
    for (operation, result) in [
        (
            DomainOperationId::TransformMotion,
            DomainType::TransformMotion,
        ),
        (
            DomainOperationId::TransformGeometry,
            DomainType::TransformGeometry,
        ),
        (DomainOperationId::Transform2d, DomainType::Transform2D),
        (DomainOperationId::VisualLayout, DomainType::VisualLayout),
        (DomainOperationId::VisualStyle, DomainType::VisualStyle),
    ] {
        let contract = registry.lookup(operation).unwrap();
        assert_eq!(contract.result(), domain(result));
        assert_eq!(
            contract.instruction(),
            DomainInstructionKind::DomainConstruct
        );
        assert_eq!(contract.effect(), Effect::Pure);
    }
}

#[test]
fn visual_attachment_is_the_only_transform_owner_mutation() {
    let registry = DomainOperationRegistry::standard();
    let update = registry.lookup(DomainOperationId::ItemWithVisual).unwrap();
    assert_eq!(update.result(), domain(DomainType::Item));
    assert_eq!(update.instruction(), DomainInstructionKind::GraphEmit);
    assert_eq!(
        update.runtime_action(),
        DomainRuntimeAction::NonOwningUpdate
    );
    assert_eq!(update.operands()[0].shape(), domain(DomainType::Item));
    assert_eq!(update.operands()[0].axis(), OperandAxis::Topology);
    assert_eq!(
        update.operands()[1].shape(),
        domain(DomainType::VisualStyle)
    );
    assert_eq!(update.operands()[1].axis(), OperandAxis::Leaf);
    assert!(registry
        .lookup_method(DomainType::Transform2D, "translated")
        .is_none());
}
