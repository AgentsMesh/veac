use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{FunctionDefinition, FunctionMap, Value};
use crate::program::model::SurfaceFile;

mod methods;

const MAX_BYTES: usize = 64 * 1024 * 1024;
pub(super) const ENTRY_BYTES: usize = 64;

pub(super) struct Budget {
    pub(super) bytes: usize,
    pub(super) limit: usize,
}

impl Budget {
    pub(super) fn function_payload_limit(&self, definitions: &[FunctionDefinition]) -> usize {
        let overhead = definitions.iter().try_fold(0usize, |bytes, definition| {
            bytes
                .checked_add(ENTRY_BYTES)?
                .checked_add(definition.name.len())
        });
        let available = self.limit.saturating_sub(self.bytes);
        overhead.map_or(0, |bytes| available.saturating_sub(bytes))
    }

    pub(super) fn value(
        &mut self,
        path: &str,
        name: &str,
        value: &Value,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.entry(path, name, value.retained_bytes(), span)
    }

    pub(super) fn functions(
        &mut self,
        file: &SurfaceFile,
        functions: &FunctionMap,
        methods: &crate::program::MethodRegistry,
    ) -> Result<(), Diagnostic> {
        let mut next = self.bytes;
        for declaration in &file.functions {
            let function = functions
                .lookup(&declaration.name)
                .expect("compiled local function must exist");
            let payload = function
                .retained_bytes()
                .ok_or_else(|| error(&file.path, declaration.span))?;
            let added = entry_bytes(&declaration.name, payload)
                .ok_or_else(|| error(&file.path, declaration.span))?;
            next = next
                .checked_add(added)
                .filter(|next| *next <= self.limit)
                .ok_or_else(|| error(&file.path, declaration.span))?;
        }
        next = methods::compiled(file, functions, methods, next, self.limit)?;
        self.bytes = next;
        Ok(())
    }

    pub(super) fn alias(
        &mut self,
        path: &str,
        qualified: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.entry(path, qualified, 0, span)
    }

    pub(super) fn type_definition(
        &mut self,
        file: &SurfaceFile,
        value: &crate::program::TypeDefinition,
    ) -> Result<(), Diagnostic> {
        let payload = value
            .retained_bytes()
            .ok_or_else(|| error(&file.path, value_span(file, value.declared_name())))?;
        self.entry(
            &file.path,
            value.declared_name(),
            payload,
            value_span(file, value.declared_name()),
        )
    }

    pub(super) fn method_definition(
        &mut self,
        file: &SurfaceFile,
        value: &crate::program::MethodDefinition,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.entry(
            &file.path,
            value.signature().name(),
            crate::program::method_system::retained_bytes(value)
                .ok_or_else(|| error(&file.path, span))?,
            span,
        )
    }

    fn entry(
        &mut self,
        path: &str,
        name: &str,
        payload: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let added = entry_bytes(name, payload).ok_or_else(|| error(path, span))?;
        self.charge(path, added, span)
    }

    fn charge(&mut self, path: &str, added: usize, span: Span) -> Result<(), Diagnostic> {
        let next = self
            .bytes
            .checked_add(added)
            .ok_or_else(|| error(path, span))?;
        if next > self.limit {
            return Err(error(path, span));
        }
        self.bytes = next;
        Ok(())
    }
}

fn entry_bytes(name: &str, payload: usize) -> Option<usize> {
    ENTRY_BYTES.checked_add(name.len())?.checked_add(payload)
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            bytes: 0,
            limit: MAX_BYTES,
        }
    }
}

pub(super) fn error(path: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_RETAINED_SCOPE_BUDGET",
        path,
        "resolved functions, constants, types, methods, and aliases exceed 64 MiB of retained symbol storage",
        span,
    )
}

fn value_span(file: &SurfaceFile, name: &str) -> Span {
    file.types
        .iter()
        .find(|value| value.name == name)
        .map(|value| value.span)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
