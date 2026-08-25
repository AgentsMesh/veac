mod bounded_cache;
mod cache;
mod core_cache;
mod dependencies;
mod dependency_query;
mod facade;
mod function_query;
mod hir_cache;
mod interface_cache;
mod interface_key;
mod interface_query;
mod key;
mod lifecycle;
mod limits;
mod revision;
mod source_graph;
mod statistics;
mod syntax_query;
mod syntax_retained;

use std::sync::{Mutex, RwLock};

use cache::SyntaxCache;
use core_cache::CoreCache;
use hir_cache::HirCache;
use interface_cache::InterfaceCache;

pub use dependencies::{CompilerDependencyRoute, CompilerDependencySnapshot};
pub(crate) use dependency_query::{
    DependencyRouteAdmission, DependencyRouteCommit, DEPENDENCY_ROUTE_ATTEMPTS,
};
pub use limits::CompilerDatabaseLimits;
pub use revision::CompilerSourceRevision;
pub use statistics::CompilerDatabaseStatistics;

#[derive(Debug)]
pub struct CompilerDatabase {
    lifecycle: RwLock<lifecycle::Lifecycle>,
    syntax: Mutex<SyntaxCache>,
    interface: Mutex<InterfaceCache>,
    hir: Mutex<HirCache>,
    core: Mutex<CoreCache>,
    dependencies: Mutex<dependencies::DependencyGraph>,
}

impl CompilerDatabase {
    pub fn with_limits(limits: CompilerDatabaseLimits) -> Self {
        Self {
            lifecycle: RwLock::new(lifecycle::Lifecycle::new()),
            syntax: Mutex::new(SyntaxCache::new(limits)),
            interface: Mutex::new(InterfaceCache::new(
                limits.interface_entries,
                limits.interface_retained_bytes,
            )),
            hir: Mutex::new(HirCache::new(limits.hir_entries, limits.hir_retained_bytes)),
            core: Mutex::new(CoreCache::new(
                limits.core_entries,
                limits.core_retained_bytes,
            )),
            dependencies: Mutex::new(dependencies::DependencyGraph::new(
                limits.dependency_entries,
                limits.dependency_retained_bytes,
            )),
        }
    }

    pub fn statistics(&self) -> CompilerDatabaseStatistics {
        let _lifecycle = self
            .lifecycle
            .write()
            .expect("compiler database lifecycle lock poisoned");
        self.collect_statistics()
    }

    fn collect_statistics(&self) -> CompilerDatabaseStatistics {
        let mut output = self
            .syntax
            .lock()
            .expect("syntax cache lock poisoned")
            .stats();
        self.interface
            .lock()
            .expect("interface cache lock poisoned")
            .contribute(&mut output);
        self.hir
            .lock()
            .expect("HIR cache lock poisoned")
            .contribute(&mut output);
        self.core
            .lock()
            .expect("core cache lock poisoned")
            .contribute(&mut output);
        let dependencies = self
            .dependencies
            .lock()
            .expect("dependency graph lock poisoned");
        dependencies.contribute(&mut output);
        output
    }

    pub fn source_revision(&self, source_id: &str, source: &str) -> CompilerSourceRevision {
        CompilerSourceRevision::new(source_id, source)
    }
}

impl Default for CompilerDatabase {
    fn default() -> Self {
        Self::with_limits(CompilerDatabaseLimits::default())
    }
}

#[cfg(test)]
#[path = "compiler_database/cache_budget_tests.rs"]
mod cache_budget_tests;
#[cfg(test)]
#[path = "compiler_database/dependency_budget_tests.rs"]
mod dependency_budget_tests;
#[cfg(test)]
#[path = "compiler_database/dependency_route_reset_tests.rs"]
mod dependency_route_reset_tests;
#[cfg(test)]
#[path = "compiler_database/dependency_tests.rs"]
mod dependency_tests;
#[cfg(test)]
#[path = "compiler_database/facade_tests.rs"]
mod facade_tests;
#[cfg(test)]
#[path = "compiler_database/lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "compiler_database/route_query_interleaving_tests.rs"]
mod route_query_interleaving_tests;
#[cfg(test)]
#[path = "compiler_database/route_query_retry_tests.rs"]
mod route_query_retry_tests;
#[cfg(test)]
#[path = "compiler_database/source_graph_authority_tests.rs"]
mod source_graph_authority_tests;
#[cfg(test)]
#[path = "compiler_database/source_graph_provenance_tests.rs"]
mod source_graph_provenance_tests;
#[cfg(test)]
#[path = "compiler_database/source_graph_tests.rs"]
mod source_graph_tests;
#[cfg(test)]
#[path = "compiler_database/stale_invalidation_admission_tests.rs"]
mod stale_invalidation_admission_tests;
#[cfg(test)]
#[path = "compiler_database/stale_route_admission_tests.rs"]
mod stale_route_admission_tests;
#[cfg(test)]
#[path = "compiler_database/statistics_barrier_tests.rs"]
mod statistics_barrier_tests;
#[cfg(test)]
#[path = "compiler_database/tests.rs"]
mod tests;
