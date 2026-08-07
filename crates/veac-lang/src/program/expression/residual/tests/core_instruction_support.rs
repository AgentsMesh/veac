use std::sync::Arc;

use crate::program::expression::core::{verify, FunctionRegistry};
use crate::program::expression::{
    BlockId, CompiledExpression, CoreBlock, CoreInstruction, CoreInstructionKind, CoreProgram,
    CoreTemporalComposeOperation as Compose, CoreTemporalProjectOperation as Project,
    CoreTerminator, CoreType, CoreTypeEntry, CoreTypeId, CoreTypeTable, CoreValueMetadata,
    PrimitiveType, Value, ValueId, ValueType, CORE_VERSION,
};
use crate::program::{DomainOperationRegistry, DomainType};

pub(super) fn raw(compose: Compose, project: Project) -> CoreProgram {
    let mut builder = Builder::default();
    let operands = compose_values(compose)
        .into_iter()
        .map(|value| builder.literal(value))
        .collect::<Vec<_>>();
    let composite = builder.compose(compose, operands);
    let result = builder.project(project, composite);
    builder.finish(result)
}

pub(super) fn compiled(compose: Compose, project: Project) -> CompiledExpression {
    let functions = FunctionRegistry::default();
    let verified = verify(raw(compose, project), &functions, &[]).unwrap();
    CompiledExpression::new(verified, Arc::new(functions))
}

#[derive(Default)]
struct Builder {
    types: Vec<CoreTypeEntry>,
    instructions: Vec<CoreInstruction>,
}

impl Builder {
    fn literal(&mut self, value: Value) -> ValueId {
        let type_id = self.intern(&value.value_type());
        self.push(
            CoreInstructionKind::Literal(value),
            type_id,
            CoreValueMetadata::constant(),
        )
    }

    fn compose(&mut self, operation: Compose, operands: Vec<ValueId>) -> ValueId {
        let value_type = compose_type(operation);
        let type_id = self.intern(&value_type);
        let metadata = operands
            .iter()
            .map(|id| self.instructions[id.index().unwrap()].metadata.clone())
            .collect::<Vec<_>>();
        self.push(
            CoreInstructionKind::TemporalCompose {
                operation,
                operands,
            },
            type_id,
            CoreValueMetadata::temporal_compose(metadata.iter()),
        )
    }

    fn project(&mut self, operation: Project, value: ValueId) -> ValueId {
        let type_id = self.intern(&project_type(operation));
        let metadata = CoreValueMetadata::temporal_project(
            &self.instructions[value.index().unwrap()].metadata,
        );
        self.push(
            CoreInstructionKind::TemporalProject { operation, value },
            type_id,
            metadata,
        )
    }

    fn push(
        &mut self,
        kind: CoreInstructionKind,
        type_id: CoreTypeId,
        metadata: CoreValueMetadata,
    ) -> ValueId {
        let id = ValueId::new(self.instructions.len() as u32);
        self.instructions.push(CoreInstruction {
            id,
            kind,
            type_id,
            metadata,
            span: id.value() as usize..id.value() as usize + 1,
        });
        id
    }

    fn intern(&mut self, value: &ValueType) -> CoreTypeId {
        if let Some(entry) = self
            .types
            .iter()
            .find(|entry| entry.kind() == &CoreType::Value(value.clone()))
        {
            return entry.id();
        }
        let id = CoreTypeId::new(self.types.len() as u32);
        self.types.push(CoreTypeEntry {
            id,
            kind: CoreType::Value(value.clone()),
        });
        id
    }

    fn finish(self, result: ValueId) -> CoreProgram {
        let domain = DomainOperationRegistry::standard();
        let result_type = self.instructions[result.index().unwrap()].type_id;
        CoreProgram {
            version: CORE_VERSION,
            domain_opset: domain.version(),
            domain_registry_digest: domain.digest(),
            entry: BlockId::new(0),
            types: CoreTypeTable::new(self.types),
            nominal_definitions: Vec::new(),
            inputs: Vec::new(),
            local_slots: Vec::new(),
            closure_definitions: Vec::new(),
            blocks: vec![CoreBlock {
                id: BlockId::new(0),
                parameters: Vec::new(),
                instructions: self.instructions,
                terminator: CoreTerminator::Return {
                    value: result,
                    span: 20..21,
                },
            }],
            result_type,
        }
    }
}

fn compose_values(value: Compose) -> Vec<Value> {
    use crate::program::expression::ExactNumber as Number;
    match value {
        Compose::Vec2 | Compose::Rect => [1, 2, 3, 4]
            .into_iter()
            .take(value.arity())
            .map(|value| Value::Scalar(Number::integer(value)))
            .collect(),
        Compose::Point => [1, 2]
            .into_iter()
            .map(|value| Value::Length(Number::integer(value)))
            .collect(),
        Compose::Color => [10, 20, 30, 40].into_iter().map(Value::Integer).collect(),
    }
}

fn compose_type(value: Compose) -> ValueType {
    match value {
        Compose::Vec2 => ValueType::domain(DomainType::Vector),
        Compose::Point => ValueType::domain(DomainType::Point),
        Compose::Rect => ValueType::domain(DomainType::Rect),
        Compose::Color => PrimitiveType::Color.into(),
    }
}

fn project_type(value: Project) -> ValueType {
    use Project::*;
    match value {
        PointX | PointY => PrimitiveType::Length.into(),
        ColorRed | ColorGreen | ColorBlue | ColorAlpha => PrimitiveType::Integer.into(),
        _ => PrimitiveType::Scalar.into(),
    }
}
