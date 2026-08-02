use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;

const MAX_VALIDATION_UNITS: usize = 16_384;
const MAX_COMPONENT_NODES: usize = 65_536;
const MAX_WORK_BYTES: usize = 256 * 1024 * 1024;

pub(crate) struct Budget {
    validation_units: usize,
    component_nodes: usize,
    work_bytes: usize,
    max_validation_units: usize,
    max_component_nodes: usize,
    max_work_bytes: usize,
}

impl Budget {
    pub(super) fn component(
        &mut self,
        path: &str,
        span: Span,
        component_nodes: usize,
        fixture_bytes: usize,
        expanded_bytes: usize,
    ) -> Result<(), Diagnostic> {
        self.record(path, span, component_nodes, fixture_bytes, expanded_bytes)
    }

    pub(super) fn preset(
        &mut self,
        path: &str,
        span: Span,
        body_bytes: usize,
        wrapped_bytes: usize,
    ) -> Result<(), Diagnostic> {
        self.record(path, span, 0, body_bytes, wrapped_bytes)
    }

    fn record(
        &mut self,
        path: &str,
        span: Span,
        component_nodes: usize,
        input_bytes: usize,
        output_bytes: usize,
    ) -> Result<(), Diagnostic> {
        let validation_units = self
            .validation_units
            .checked_add(1)
            .ok_or_else(|| limit(path, span, "validation unit count overflowed"))?;
        if validation_units > self.max_validation_units {
            return Err(limit(
                path,
                span,
                format!(
                    "definition validation exceeds {MAX_VALIDATION_UNITS} bounded validation units"
                ),
            ));
        }
        let component_nodes = self
            .component_nodes
            .checked_add(component_nodes)
            .ok_or_else(|| limit(path, span, "component node count overflowed"))?;
        if component_nodes > self.max_component_nodes {
            return Err(limit(
                path,
                span,
                format!(
                    "component definition validation exceeds {MAX_COMPONENT_NODES} expanded component nodes"
                ),
            ));
        }
        let work = input_bytes
            .checked_add(output_bytes)
            .and_then(|bytes| self.work_bytes.checked_add(bytes))
            .ok_or_else(|| limit(path, span, "work byte count overflowed"))?;
        if work > self.max_work_bytes {
            return Err(limit(
                path,
                span,
                format!(
                    "definition validation exceeds {} MiB of source work",
                    MAX_WORK_BYTES / (1024 * 1024)
                ),
            ));
        }
        self.validation_units = validation_units;
        self.component_nodes = component_nodes;
        self.work_bytes = work;
        Ok(())
    }

    pub(crate) fn with_limits(
        max_validation_units: usize,
        max_component_nodes: usize,
        max_work_bytes: usize,
    ) -> Self {
        Self {
            validation_units: 0,
            component_nodes: 0,
            work_bytes: 0,
            max_validation_units,
            max_component_nodes,
            max_work_bytes,
        }
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::with_limits(MAX_VALIDATION_UNITS, MAX_COMPONENT_NODES, MAX_WORK_BYTES)
    }
}

fn limit(path: &str, span: Span, detail: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PROGRAM_DEFINITION_BUDGET", path, detail, span)
}

#[cfg(test)]
mod tests;
