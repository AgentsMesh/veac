use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::Value;
use crate::program::model::{ComponentKey, SurfaceFile};
use std::collections::BTreeMap;

mod component;

const MAX_BYTES: usize = 64 * 1024 * 1024;
pub(super) const ENTRY_BYTES: usize = 64;

pub(super) struct Budget {
    pub(super) bytes: usize,
    pub(super) limit: usize,
}

impl Budget {
    pub(super) fn value(
        &mut self,
        path: &str,
        name: &str,
        value: &Value,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.entry(path, name, value.retained_bytes(), span)
    }

    pub(super) fn preset(
        &mut self,
        path: &str,
        name: &str,
        body: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.entry(path, name, body.len(), span)
    }

    pub(super) fn alias(
        &mut self,
        path: &str,
        qualified: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.entry(path, qualified, 0, span)
    }

    pub(super) fn components(
        &mut self,
        file: &SurfaceFile,
        captured: &BTreeMap<String, ComponentKey>,
    ) -> Result<(), Diagnostic> {
        let Some(first) = file.components.first() else {
            return Ok(());
        };
        let added = component::logical_bytes(file, captured)
            .ok_or_else(|| error(&file.path, first.span))?;
        self.charge(&file.path, added, first.span)
    }

    fn entry(
        &mut self,
        path: &str,
        name: &str,
        payload: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let added = ENTRY_BYTES
            .checked_add(name.len())
            .and_then(|bytes| bytes.checked_add(payload))
            .ok_or_else(|| error(path, span))?;
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

impl Default for Budget {
    fn default() -> Self {
        Self {
            bytes: 0,
            limit: MAX_BYTES,
        }
    }
}

fn error(path: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_RETAINED_SCOPE_BUDGET",
        path,
        "resolved constants, presets, components, and aliases exceed 64 MiB of retained symbol storage",
        span,
    )
}

#[cfg(test)]
mod tests;
