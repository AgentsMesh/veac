use super::control::Flow;
use super::iterable::Iterable;
use super::slot::{public_at, RuntimeValue};
use super::Evaluator;
use crate::program::expression::core::{CoreForEach, VerifiedCoreProgram};
use crate::program::expression::{
    CollectionOperation, ExecutionFrame, ExpressionError, Value, MAX_FUNCTION_CALL_DEPTH,
};

impl Evaluator<'_> {
    pub(super) fn for_each(
        &mut self,
        loop_value: &CoreForEach,
        program: &VerifiedCoreProgram,
        values: &mut [Option<RuntimeValue>],
        call_depth: usize,
    ) -> Result<Flow, ExpressionError> {
        let span = loop_value.provenance().loop_span().clone();
        self.enter(span.clone())?;
        if call_depth >= MAX_FUNCTION_CALL_DEPTH {
            return Err(ExpressionError::new(
                "EXPRESSION_CALL_DEPTH_LIMIT",
                format!("function calls exceed the {MAX_FUNCTION_CALL_DEPTH} depth limit"),
                span,
            ));
        }
        let mut iterable = Iterable::new(
            public_at(values, loop_value.iterable(), &span)?,
            span.clone(),
        )?;
        if iterable.count() > u64::from(loop_value.maximum_count()) {
            return Err(contract(
                "for-each input exceeds its verified static bound",
                span,
            ));
        }
        let count = self.execution.reserve_aggregate(
            CollectionOperation::Map,
            iterable.is_map(),
            iterable.count(),
            span.clone(),
        )?;
        let captures = loop_value
            .captures()
            .iter()
            .map(|id| public_at(values, *id, &span))
            .collect::<Result<Vec<_>, _>>()?;
        self.execution
            .reserve_closure(captures.len(), span.clone())?;
        let definition = program
            .closure(loop_value.body())
            .cloned()
            .ok_or_else(|| contract("for-each body is unavailable", span.clone()))?;
        let provenance = self.frames.last().map(|frame| {
            frame
                .definition()
                .closure(definition.digest(), span.clone())
        });
        let mut output = Vec::with_capacity(count);
        let mut index = 0usize;
        while let Some(element) = iterable.next(span.clone())? {
            let loop_frame = self.frames.last().map(|frame| {
                frame.iteration(
                    span.clone(),
                    loop_value.provenance().binding_span().clone(),
                    index,
                )
            });
            let pushed_loop_frame = loop_frame.is_some();
            if let Some(frame) = loop_frame {
                self.iterations.push(frame);
            }
            let frame = provenance
                .clone()
                .map(|definition| ExecutionFrame::new(definition, self.origin(span.clone())));
            let result = self.program(
                definition.body(),
                &[element, Value::Integer(index as i64)],
                &captures,
                call_depth + 1,
                None,
                frame,
            );
            let frames = self.iterations.clone();
            if pushed_loop_frame {
                self.iterations.pop();
            }
            output.push(result.map_err(|error| error.in_loop_frames(&frames))?);
            index += 1;
        }
        let result = Value::list(definition.body().core().result_type().clone(), output)
            .map_err(|_| contract("verified for-each result construction failed", span.clone()))?;
        let target = &program.core().blocks[loop_value
            .continuation()
            .index()
            .expect("verified continuation")];
        let parameter = &target.parameters[0];
        values[parameter.id.index().expect("verified result parameter")] =
            Some(RuntimeValue::Public(result));
        Ok(Flow::Continue(loop_value.continuation()))
    }
}

fn contract(message: &str, span: std::ops::Range<usize>) -> ExpressionError {
    ExpressionError::new("EXPRESSION_RUNTIME_CONTRACT", message, span)
}
