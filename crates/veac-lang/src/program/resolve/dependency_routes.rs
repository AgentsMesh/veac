use std::collections::BTreeMap;

use super::{Resolver, SurfaceFile};
use crate::authoring::Span;
use crate::program::compiler_database::DependencyRouteCommit;
use crate::program::diagnostic::Diagnostic;

impl Resolver<'_> {
    pub(super) fn commit_dependency_routes(
        &mut self,
        file: &SurfaceFile,
        routes: BTreeMap<String, String>,
    ) {
        let mut admission = self
            .route_admissions
            .get(&file.path)
            .expect("parsed file has dependency route admission")
            .clone();
        if self.database.commit_routes(&mut admission, routes)
            == DependencyRouteCommit::RetryRequired
        {
            self.retry_required = true;
        }
        self.route_admissions.insert(file.path.clone(), admission);
    }
}

pub(super) fn query_changed(path: &str) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        "PROGRAM_QUERY_CHANGED",
        path,
        "source dependency state changed during every bounded query attempt",
        Span::default(),
    )]
}
