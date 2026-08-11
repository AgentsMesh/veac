use super::{EntryContract, EntryValueType};
use crate::authoring::Span;
use crate::program::expression::{ValueType, ValueTypeKind};
use crate::program::model::SurfaceFile;
use crate::program::{Diagnostic, TypeDefinitionKind, TypeRegistry};

impl EntryContract {
    pub(crate) fn validate_surface(&self, file: &SurfaceFile) -> Result<(), Diagnostic> {
        if self.legacy_video {
            return crate::program::executable::validate_entry(file);
        }
        let mut entries = file
            .functions
            .iter()
            .filter(|value| value.name == self.function);
        let entry = entries.next().ok_or_else(|| {
            error(
                file,
                "PROGRAM_ENTRY_MISSING",
                "host entry function is missing",
            )
        })?;
        if entries.next().is_some() {
            return Err(error(
                file,
                "PROGRAM_ENTRY_DUPLICATE",
                "host entry function is duplicated",
            ));
        }
        if entry.parameters.len() != self.parameters.len() {
            return Err(at(
                file,
                "PROGRAM_ENTRY_ARITY",
                "host entry has the wrong arity",
                entry.span,
            ));
        }
        if !self.allow_inputs && !file.inputs.is_empty() {
            return Err(error(
                file,
                "PROGRAM_ENTRY_INPUTS",
                "host entry forbids build inputs",
            ));
        }
        if !self.allow_temporal && !file.temporal.is_empty() {
            return Err(error(
                file,
                "PROGRAM_ENTRY_TEMPORAL",
                "host entry forbids Temporal declarations",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_compiled(
        &self,
        function: &crate::program::expression::CompiledFunction,
        types: &TypeRegistry,
    ) -> Result<(), Diagnostic> {
        let valid_parameters =
            function
                .parameters()
                .iter()
                .zip(&self.parameters)
                .all(|(actual, expected)| {
                    actual.name == expected.name
                        && expected.value_type.matches(&actual.value_type, types)
                });
        if function.parameters().len() != self.parameters.len()
            || !valid_parameters
            || !self.result.matches(function.return_type(), types)
        {
            return Err(compiled_error(
                function,
                "PROGRAM_ENTRY_SIGNATURE",
                "compiled entry signature does not match its host contract",
            ));
        }
        let summary = function.summary();
        if summary.effect() > self.max_effect
            || summary.result().shape_stage() > self.max_stage
            || summary.result().leaf_stage() > self.max_stage
        {
            return Err(compiled_error(
                function,
                "PROGRAM_ENTRY_EFFECT",
                "entry exceeds its host effect or stage ceiling",
            ));
        }
        Ok(())
    }
}

impl EntryValueType {
    fn matches(&self, value: &ValueType, types: &TypeRegistry) -> bool {
        match (self, value.kind()) {
            (Self::Primitive(expected), ValueTypeKind::Primitive(actual)) => expected == &actual,
            (Self::Domain(expected), ValueTypeKind::Domain(actual)) => expected == &actual,
            (
                Self::Nominal {
                    source_id,
                    declared_name,
                },
                ValueTypeKind::Nominal(actual),
            ) => types.definition(actual.id()).is_some_and(|definition| {
                matches!(
                    definition.kind(),
                    TypeDefinitionKind::Struct(_) | TypeDefinitionKind::Enum(_)
                ) && definition.declared_name() == declared_name
                    && source_id
                        .as_deref()
                        .is_none_or(|id| definition.canonical_source_id() == id)
            }),
            _ => false,
        }
    }
}

fn error(file: &SurfaceFile, code: &'static str, message: &'static str) -> Diagnostic {
    at(file, code, message, Span::default())
}

fn at(file: &SurfaceFile, code: &'static str, message: &'static str, span: Span) -> Diagnostic {
    Diagnostic::new(code, &file.path, message, span)
}

fn compiled_error(
    function: &crate::program::expression::CompiledFunction,
    code: &'static str,
    message: &'static str,
) -> Diagnostic {
    let range = function.origin().map_or(0..0, |value| value.body_span());
    Diagnostic::new(
        code,
        function
            .origin()
            .map_or("<entry>", |value| value.source_id()),
        message,
        Span {
            start: range.start,
            end: range.end,
        },
    )
}
