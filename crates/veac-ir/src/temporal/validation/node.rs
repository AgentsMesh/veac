use std::collections::BTreeMap;

use crate::temporal::{
    TemporalInputDeclaration, TemporalNode, TemporalNodeId, TemporalNodeKind, TemporalProgram,
    TemporalType,
};

use super::{operation, Validator};

impl Validator {
    pub(super) fn nodes(&mut self, program: &TemporalProgram, pointer: &str) {
        self.charge_nodes(program.nodes.len(), &format!("{pointer}/nodes"));
        if program.nodes.is_empty() || program.nodes.len() > super::program::per_program_limit() {
            self.push(
                "TEMPORAL_PROGRAM_NODE_COUNT",
                format!("{pointer}/nodes"),
                "program node count is outside the contract",
            );
        }
        let inputs = program
            .inputs
            .iter()
            .map(|input| (input.id.get(), input))
            .collect();
        let mut types = Vec::with_capacity(program.nodes.len());
        for (index, node) in program.nodes.iter().enumerate() {
            let path = format!("{pointer}/nodes/{index}");
            if node.id.get() != u32::try_from(index).unwrap_or(u32::MAX) {
                self.push(
                    "TEMPORAL_NODE_ORDER",
                    format!("{path}/id"),
                    "node IDs must be contiguous and ordered",
                );
            }
            let inferred = self.node(node, &types, &inputs, &path);
            if matches!(inferred, Some(value) if value != node.value_type) {
                self.push(
                    "TEMPORAL_NODE_TYPE",
                    format!("{path}/value_type"),
                    "node result type does not match its operation",
                );
            }
            types.push(node.value_type);
        }
        let result = match usize::try_from(program.result.get()) {
            Ok(index) => types.get(index),
            Err(_) => None,
        };
        if result != Some(&program.result_type) {
            self.push(
                "TEMPORAL_PROGRAM_RESULT",
                format!("{pointer}/result"),
                "program result is missing or has the wrong type",
            );
        }
    }

    fn node(
        &mut self,
        node: &TemporalNode,
        types: &[TemporalType],
        inputs: &BTreeMap<u32, &TemporalInputDeclaration>,
        pointer: &str,
    ) -> Option<TemporalType> {
        if matches!(&node.provenance_id, Some(id) if !id.is_valid()) {
            self.push(
                "TEMPORAL_PROVENANCE_ID",
                format!("{pointer}/provenance_id"),
                "invalid node provenance ID",
            );
        }
        match &node.kind {
            TemporalNodeKind::Literal { value } => {
                self.value(value, &format!("{pointer}/value"));
                Some(value.value_type())
            }
            TemporalNodeKind::Input { input_id } => match inputs.get(&input_id.get()) {
                Some(value) => Some(value.value_type),
                None => {
                    self.push(
                        "TEMPORAL_INPUT_REFERENCE",
                        format!("{pointer}/input_id"),
                        "node references an undeclared input",
                    );
                    None
                }
            },
            TemporalNodeKind::Unary {
                operation: op,
                operand,
            } => {
                let operand = self.operand(*operand, types, pointer)?;
                self.typed_operation(operation::unary(*op, operand), pointer)
            }
            TemporalNodeKind::Binary {
                operation: op,
                left,
                right,
            } => {
                let left = self.operand(*left, types, pointer)?;
                let right = self.operand(*right, types, pointer)?;
                self.typed_operation(operation::binary(*op, left, right), pointer)
            }
            TemporalNodeKind::Compare {
                operation: op,
                left,
                right,
            } => {
                let left = self.operand(*left, types, pointer)?;
                let right = self.operand(*right, types, pointer)?;
                self.typed_operation(operation::compare(*op, left, right), pointer)
            }
            TemporalNodeKind::Select {
                condition,
                when_true,
                when_false,
            } => {
                let condition = self.operand(*condition, types, pointer)?;
                let left = self.operand(*when_true, types, pointer)?;
                let right = self.operand(*when_false, types, pointer)?;
                if condition == TemporalType::Boolean && left == right {
                    Some(left)
                } else {
                    self.bad_operation(pointer)
                }
            }
            TemporalNodeKind::CurveSample { input, keys } => {
                let input = self.operand(*input, types, pointer)?;
                self.curve(input, keys, pointer)
            }
            kind @ (TemporalNodeKind::ComposeVec2 { .. }
            | TemporalNodeKind::ProjectVec2 { .. }
            | TemporalNodeKind::ComposePoint { .. }
            | TemporalNodeKind::ProjectPoint { .. }
            | TemporalNodeKind::ComposeRect { .. }
            | TemporalNodeKind::ProjectRect { .. }
            | TemporalNodeKind::ComposeColor { .. }
            | TemporalNodeKind::ProjectColor { .. }) => self.composite(kind, types, pointer),
        }
    }

    pub(super) fn operand(
        &mut self,
        id: TemporalNodeId,
        types: &[TemporalType],
        pointer: &str,
    ) -> Option<TemporalType> {
        let value = match usize::try_from(id.get()) {
            Ok(index) => types.get(index).copied(),
            Err(_) => None,
        };
        match value {
            Some(value) => Some(value),
            None => {
                self.push(
                    "TEMPORAL_NODE_REFERENCE",
                    pointer,
                    "operand must reference an earlier node",
                );
                None
            }
        }
    }

    fn typed_operation(
        &mut self,
        value: Option<TemporalType>,
        pointer: &str,
    ) -> Option<TemporalType> {
        match value {
            Some(value) => Some(value),
            None => self.bad_operation(pointer),
        }
    }

    pub(super) fn bad_operation(&mut self, pointer: &str) -> Option<TemporalType> {
        self.push(
            "TEMPORAL_OPERATION_TYPE",
            pointer,
            "operation operand types are incompatible",
        );
        None
    }
}
