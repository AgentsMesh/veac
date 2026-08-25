use veac_lang_model::{Effect, PrimitiveType, Stage};

use super::*;

#[path = "tests/action.rs"]
mod action_tests;
#[path = "tests/catalog.rs"]
mod catalog_tests;
#[path = "tests/contracts.rs"]
mod contract_tests;
#[path = "tests/exposure.rs"]
mod exposure_tests;
#[path = "tests/identity.rs"]
mod identity_tests;
#[path = "tests/registry.rs"]
mod registry_tests;
#[path = "tests/relation.rs"]
mod relation_tests;
#[path = "tests/runtime_action.rs"]
mod runtime_action_tests;
#[path = "tests/surface.rs"]
mod surface_tests;
#[path = "tests/temporal.rs"]
mod temporal_tests;
#[path = "tests/transform.rs"]
mod transform_tests;
#[path = "tests/validation.rs"]
mod validation_tests;
#[path = "tests/version.rs"]
mod version_tests;

fn all_contracts() -> Vec<DomainOperationContract> {
    super::catalog::contracts()
}

fn builtin(id: DomainOperationId) -> DomainOperationContract {
    super::catalog::contract(id)
}

fn rebuild(
    values: impl IntoIterator<Item = DomainOperationContract>,
    count_limit: usize,
    retained_limit: usize,
) -> Result<DomainOperationRegistry, DomainRegistryError> {
    super::registry::build(
        DomainOpsetVersion::CURRENT,
        values,
        count_limit,
        retained_limit,
    )
}

fn operation(
    id: DomainOperationId,
    name: &str,
    operands: Vec<DomainOperandContract>,
    result: DomainValueShape,
    instruction: DomainInstructionKind,
    effect: Effect,
) -> DomainOperationContract {
    let runtime_action = match instruction {
        DomainInstructionKind::DomainConstruct => DomainRuntimeAction::Description,
        DomainInstructionKind::GraphEmit => DomainRuntimeAction::EntityConstructor,
    };
    operation_with_exposure(
        id,
        name,
        DomainOperationExposure::free_function(name),
        operands,
        result,
        DomainOperationSemantics::new(instruction, runtime_action, effect, Stage::Build, None),
    )
}

fn operation_with_exposure(
    id: DomainOperationId,
    name: &str,
    exposure: DomainOperationExposure,
    operands: Vec<DomainOperandContract>,
    result: DomainValueShape,
    semantics: DomainOperationSemantics,
) -> DomainOperationContract {
    DomainOperationContract::new(id, name, exposure, operands, result, semantics)
}

fn operand(name: &str, shape: DomainValueShape, axis: OperandAxis) -> DomainOperandContract {
    DomainOperandContract::new(name, shape, axis)
}

fn primitive(value: PrimitiveType) -> DomainValueShape {
    DomainValueShape::primitive(value)
}

fn domain(value: DomainType) -> DomainValueShape {
    DomainValueShape::domain(value)
}
