use std::sync::Arc;

use crate::authoring::Span;
use crate::program::build_input::VerifiedBuildInputs;
use crate::program::diagnostic::{Diagnostic, Diagnostics};
use crate::program::expression::{self, ExecutionBudget};
use crate::program::BuildInputManifestV1;

use super::{BuiltProgram, ExecutableBuild};

impl ExecutableBuild {
    pub fn execute(&self) -> Result<BuiltProgram, Diagnostics> {
        self.execute_with_inputs(&BuildInputManifestV1::empty())
    }

    pub fn execute_with_inputs(
        &self,
        manifest: &BuildInputManifestV1,
    ) -> Result<BuiltProgram, Diagnostics> {
        let inputs = self.bind_inputs(manifest)?;
        self.execute_verified(&inputs)
    }

    pub fn validate_inputs(&self, manifest: &BuildInputManifestV1) -> Result<(), Diagnostics> {
        self.bind_inputs(manifest).map(|_| ())
    }

    fn bind_inputs(
        &self,
        manifest: &BuildInputManifestV1,
    ) -> Result<VerifiedBuildInputs, Diagnostics> {
        VerifiedBuildInputs::bind(&self.build_inputs, &self.types, manifest).map_err(|error| {
            Diagnostics::one(Diagnostic::new(
                error.code(),
                self.root_module(),
                error.message(),
                Span::default(),
            ))
        })
    }

    fn execute_verified(&self, inputs: &VerifiedBuildInputs) -> Result<BuiltProgram, Diagnostics> {
        self.execute_verified_with_ledger(inputs, &ExecutionBudget::default())
    }

    pub(super) fn execute_verified_with_ledger(
        &self,
        inputs: &VerifiedBuildInputs,
        ledger: &ExecutionBudget,
    ) -> Result<BuiltProgram, Diagnostics> {
        let fallback = self.fallback();
        let declared_inputs = super::super::temporal::declared_inputs(
            &self.main,
            &self.temporal_leaves,
            inputs.digest(),
        );
        let identity =
            super::super::identity::read(&self.main, self.source_graph(), declared_inputs);
        ledger.transaction(|| {
            let graph = expression::runtime::execute_entry(
                &self.main,
                &self.functions,
                ledger,
                identity,
                inputs,
            )
            .map_err(|error| self.runtime_error(fallback, error))?;
            let envelope = super::super::lower::project_with_inputs(
                &graph,
                &self.temporal_leaves,
                inputs.residual_bindings(),
                ledger,
            )
            .map_err(|error| self.lower_error(fallback, error))?;
            Ok(BuiltProgram {
                source_graph: self.source_graph.clone(),
                main: Arc::clone(&self.main),
                methods: Arc::clone(&self.methods),
                types: Arc::clone(&self.types),
                build_inputs: Arc::clone(&self.build_inputs),
                graph,
                envelope,
            })
        })
    }

    fn fallback(&self) -> Span {
        self.main
            .origin()
            .map(|origin| span(origin.body_span()))
            .unwrap_or_default()
    }

    fn runtime_error(&self, fallback: Span, error: expression::ExpressionError) -> Diagnostics {
        Diagnostics::one(super::super::super::expression_diagnostic::runtime(
            "PROGRAM_EXECUTABLE_RUNTIME",
            self.root_module(),
            fallback,
            error,
        ))
    }

    fn lower_error(
        &self,
        fallback: Span,
        error: super::super::lower::ExecutableLowerError,
    ) -> Diagnostics {
        Diagnostics::one(Diagnostic::new(
            error.diagnostic_code(),
            self.root_module(),
            format!("{}: {error}", error.reason_code()),
            fallback,
        ))
    }
}

fn span(value: std::ops::Range<usize>) -> Span {
    Span {
        start: value.start,
        end: value.end,
    }
}
