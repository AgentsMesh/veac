use std::collections::BTreeSet;

use super::super::expression::{CoreInstructionKind, CoreProgram, FunctionId, FunctionParameter};
use super::super::model::Scope;
use super::super::{Diagnostics, DomainOperationRegistry};
use super::model::ModuleDomainCapability;

pub(super) fn collect(
    source_id: &str,
    scope: &Scope,
) -> Result<Vec<ModuleDomainCapability>, Diagnostics> {
    let mut pending = scope
        .functions
        .iter()
        .map(|(_, function)| function.id())
        .chain(
            scope
                .methods
                .definitions()
                .filter(|method| method.owner_source() == source_id)
                .map(|method| method.signature().function_id()),
        )
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    let mut opcodes = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let function = scope
            .functions
            .lookup_by_id(id)
            .expect("exported implementation closure is retained");
        scan(function.body(), &mut opcodes, &mut pending);
        defaults(function.parameters(), &mut pending);
    }
    let registry = DomainOperationRegistry::standard();
    Ok(opcodes
        .into_iter()
        .map(|opcode| {
            let contract = registry
                .lookup_opcode(opcode)
                .expect("verified Core contains a current Domain opcode");
            ModuleDomainCapability {
                opcode,
                name: contract.name().to_owned(),
            }
        })
        .collect())
}

fn scan(program: &CoreProgram, opcodes: &mut BTreeSet<u16>, pending: &mut Vec<FunctionId>) {
    for instruction in program
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
    {
        match instruction.kind() {
            CoreInstructionKind::DomainConstruct { opcode, .. }
            | CoreInstructionKind::GraphEmit { opcode, .. } => {
                opcodes.insert(*opcode);
            }
            _ => {}
        }
    }
    for closure in program.closure_definitions() {
        scan(closure.body(), opcodes, pending);
    }
    pending.extend(program.called_functions());
}

fn defaults(parameters: &[FunctionParameter], pending: &mut Vec<FunctionId>) {
    pending.extend(
        parameters
            .iter()
            .filter_map(|parameter| parameter.default().map(|value| value.thunk())),
    );
}
