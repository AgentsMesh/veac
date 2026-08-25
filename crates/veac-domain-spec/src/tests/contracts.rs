use super::*;

#[test]
fn every_v8_operation_contract_is_self_consistent() {
    let registry = DomainOperationRegistry::standard();
    for contract in registry.contracts() {
        assert_eq!(contract.name(), contract.id().name());
        assert!(contract.result().is_single_domain());
        match contract.runtime_action() {
            DomainRuntimeAction::Description => {
                assert_eq!(
                    contract.instruction(),
                    DomainInstructionKind::DomainConstruct
                )
            }
            _ => {
                assert_eq!(contract.instruction(), DomainInstructionKind::GraphEmit);
                assert!(contract.result().domain_type().unwrap().is_graph_entity());
            }
        }
        for operand in contract.operands() {
            assert!(!operand.name().is_empty());
            if operand
                .shape()
                .domain_type()
                .is_some_and(DomainType::requires_topology_axis)
            {
                assert_eq!(operand.axis(), OperandAxis::Topology, "{}", contract.name());
            }
        }
    }
}

#[test]
fn representative_family_snapshots_are_exact() {
    let registry = DomainOperationRegistry::standard();
    let expected = [
        "canvas(width:Primitive(Length)@Topology,height:Primitive(Length)@Topology)->Domain(Canvas):DomainConstruct/Pure",
        "project(key:Primitive(Identifier)@Topology,settings:Domain(ProjectSettings)@Topology)->Domain(Project):GraphEmit/GraphEmit",
        "source_generated(generator:Domain(Generator)@Topology)->Domain(Source):DomainConstruct/Pure",
        "item_with_visual(item:Domain(Item)@Topology,style:Domain(VisualStyle)@Leaf)->Domain(Item):GraphEmit/GraphEmit",
        "sequence_with_relation(sequence:Domain(Sequence)@Topology,relation:Domain(Relation)@Topology)->Domain(Sequence):GraphEmit/GraphEmit",
        "project_with_delivery(project:Domain(Project)@Topology,delivery:Domain(Delivery)@Topology)->Domain(Project):GraphEmit/GraphEmit",
    ];
    let operations = [
        DomainOperationId::Canvas,
        DomainOperationId::Project,
        DomainOperationId::SourceGenerated,
        DomainOperationId::ItemWithVisual,
        DomainOperationId::SequenceWithRelation,
        DomainOperationId::ProjectWithDelivery,
    ];
    let actual = operations.map(|id| snapshot(registry.lookup(id).unwrap()));
    assert_eq!(actual, expected);
}

fn snapshot(value: &DomainOperationContract) -> String {
    let operands = value
        .operands()
        .iter()
        .map(|operand| {
            format!(
                "{}:{:?}@{:?}",
                operand.name(),
                operand.shape(),
                operand.axis()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}({operands})->{:?}:{:?}/{:?}",
        value.name(),
        value.result(),
        value.instruction(),
        value.effect()
    )
}
