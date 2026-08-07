use super::Builder;
use crate::program::expression::core::{
    ClosureDefinitionId, CoreClosureDefinition, CoreInstructionKind, FunctionSummary, ValueId,
};
use crate::program::expression::hir::{TypedBlock, TypedCapture, TypedClosureParameter, TypedNode};
use crate::program::expression::{ValueType, ValueTypeKind};

pub(super) struct DefinedClosure {
    pub(super) id: ClosureDefinitionId,
    pub(super) captures: Vec<ValueId>,
    pub(super) summary: FunctionSummary,
}

impl Builder<'_> {
    pub(super) fn closure(
        &mut self,
        parameters: &[TypedClosureParameter],
        captures: &[TypedCapture],
        body: &TypedBlock,
        non_escaping: bool,
        node: &TypedNode,
    ) -> crate::program::expression::ValueId {
        let defined = self.define_closure(parameters, &[], captures, body, non_escaping, node);
        self.emit(
            CoreInstructionKind::Closure {
                definition: defined.id,
                captures: defined.captures,
            },
            &node.value_type,
            node.span.clone(),
        )
    }

    pub(super) fn define_closure(
        &mut self,
        parameters: &[TypedClosureParameter],
        extra_parameters: &[ValueType],
        captures: &[TypedCapture],
        body: &TypedBlock,
        non_escaping: bool,
        node: &TypedNode,
    ) -> DefinedClosure {
        let capture_values = captures
            .iter()
            .map(|capture| self.node(&capture.source))
            .collect::<Vec<_>>();
        let capture_types = captures
            .iter()
            .map(|capture| capture.source.value_type.clone())
            .collect::<Vec<_>>();
        let mut parameter_types = parameters
            .iter()
            .map(|parameter| parameter.value_type.clone())
            .collect::<Vec<_>>();
        parameter_types.extend_from_slice(extra_parameters);
        let mut parameter_stages = parameters
            .iter()
            .map(|parameter| parameter.stage)
            .collect::<Vec<_>>();
        parameter_stages.extend(std::iter::repeat_n(
            crate::program::expression::Stage::Const,
            extra_parameters.len(),
        ));
        let mut child = Builder::new(
            self.registry,
            self.trusted_functions,
            self.callable_inputs,
            self.input_identity,
            self.nominal_types,
            self.domain,
        );
        child.prepare_block(body, &parameter_types, &parameter_stages, &capture_types);
        let result = child.typed_block(body);
        let extra_types = parameter_types
            .iter()
            .chain(&capture_types)
            .cloned()
            .collect::<Vec<_>>();
        let program = child.finish(
            result,
            &body.result.value_type,
            body.result.span.clone(),
            &extra_types,
        );
        let summary = program.function_summary();
        let ValueTypeKind::Function { effect, .. } = node.value_type.kind() else {
            unreachable!("closure node has a function type")
        };
        let digest = crate::program::expression::core::closure_digest(
            &parameter_types,
            &parameter_stages,
            &capture_types,
            effect,
            non_escaping,
            &program,
        );
        let definition = ClosureDefinitionId::new(
            u32::try_from(self.closure_definitions.len())
                .expect("closure definition limit fits u32"),
        );
        self.closure_definitions.push(CoreClosureDefinition {
            id: definition,
            parameter_types,
            parameter_stages,
            capture_types,
            effect,
            non_escaping,
            body: Box::new(program),
            summary: summary.clone(),
            digest,
            span: node.span.clone(),
        });
        DefinedClosure {
            id: definition,
            captures: capture_values,
            summary,
        }
    }
}
