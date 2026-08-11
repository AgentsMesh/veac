use crate::{
    BuildAction, BuildCache, BuildLimits, BuildReceipt, BuildResult, CancellationToken,
    NodeExecutor, ValidatedGraph,
};

mod completion;
mod prepare;
mod run;
mod state;

#[derive(Debug, Clone, Copy)]
pub struct BuildScheduler {
    limits: BuildLimits,
}

impl BuildScheduler {
    pub fn new(limits: BuildLimits) -> Self {
        Self { limits }
    }

    pub fn limits(&self) -> BuildLimits {
        self.limits
    }

    pub fn run<A>(
        &self,
        graph: &ValidatedGraph<A>,
        executor: &dyn NodeExecutor<A>,
        cache: &dyn BuildCache,
        cancellation: CancellationToken,
    ) -> BuildResult<BuildReceipt>
    where
        A: BuildAction,
    {
        run::execute(graph, executor, cache, cancellation, self.limits)
    }
}
