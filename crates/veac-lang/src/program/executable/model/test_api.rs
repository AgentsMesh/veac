use super::{ExecutableBuild, ExecutableTemporalLeaf};
use crate::program::build_input::VerifiedBuildInputs;
use crate::program::expression::ExecutionBudget;
use crate::program::{BuildInputManifestV1, BuiltProgram, Diagnostics};

impl ExecutableBuild {
    pub(crate) fn with_temporal_leaf(mut self, leaf: ExecutableTemporalLeaf) -> Self {
        self.temporal_leaves.push(leaf);
        self
    }

    pub(crate) fn push_temporal_leaf(&mut self, leaf: ExecutableTemporalLeaf) {
        self.temporal_leaves.push(leaf);
    }

    pub(crate) fn temporal_leaves(&self) -> &[ExecutableTemporalLeaf] {
        &self.temporal_leaves
    }

    pub(crate) fn execute_with_ledger(
        &self,
        ledger: &ExecutionBudget,
    ) -> Result<BuiltProgram, Diagnostics> {
        let inputs = VerifiedBuildInputs::bind(
            &self.build_inputs,
            &self.types,
            &BuildInputManifestV1::empty(),
        )
        .expect("prepared executable build has valid empty inputs");
        self.execute_verified_with_ledger(&inputs, ledger)
    }
}
