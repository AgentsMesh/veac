mod diagnostics;

use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::limits::SourceBudget;
use crate::program::loader::{validate_source_id, LoadedSource, SourceAuthority, SourceLoader};
use crate::program::model::SurfaceFile;
use crate::program::PreparedSourceGraph;

use super::super::{
    CompilerDatabase, DependencyRouteAdmission, DependencyRouteCommit, DEPENDENCY_ROUTE_ATTEMPTS,
};
use diagnostics::{authority_collision, check_active, source_collision, source_id};

pub(super) fn run(
    database: &CompilerDatabase,
    root: LoadedSource,
    loader: &dyn SourceLoader,
    root_imports: &[String],
) -> Result<(PreparedSourceGraph, DependencyRouteAdmission), Vec<Diagnostic>> {
    validate_source_id(&root.id).map_err(|message| vec![source_id(&root.id, message)])?;
    for _ in 0..DEPENDENCY_ROUTE_ATTEMPTS {
        let (graph, admission, retry) = attempt(database, root.clone(), loader, root_imports)?;
        if !retry {
            return Ok((graph, admission));
        }
    }
    Err(vec![diagnostics::query_changed(&root.id)])
}

fn attempt(
    database: &CompilerDatabase,
    root: LoadedSource,
    loader: &dyn SourceLoader,
    root_imports: &[String],
) -> Result<(PreparedSourceGraph, DependencyRouteAdmission, bool), Vec<Diagnostic>> {
    let root_module = root.id.clone();
    let mut state = Discovery::default();
    let mut context = VisitContext {
        database,
        loader,
        budget: SourceBudget::default(),
        active: Vec::new(),
    };
    let admission = state
        .visit(&mut context, root, root_imports)?
        .expect("root source is not already discovered");
    Ok((
        PreparedSourceGraph::new(
            root_module,
            state.sources,
            state.authorities,
            state.resolutions,
        ),
        admission,
        state.retry_required,
    ))
}

struct VisitContext<'a> {
    database: &'a CompilerDatabase,
    loader: &'a dyn SourceLoader,
    budget: SourceBudget,
    active: Vec<String>,
}

#[derive(Default)]
struct Discovery {
    sources: BTreeMap<String, String>,
    authorities: BTreeMap<String, SourceAuthority>,
    resolutions: BTreeMap<(String, String), String>,
    retry_required: bool,
}

impl Discovery {
    fn visit(
        &mut self,
        context: &mut VisitContext<'_>,
        loaded: LoadedSource,
        implicit: &[String],
    ) -> Result<Option<DependencyRouteAdmission>, Vec<Diagnostic>> {
        check_active(&context.active, &loaded.id)?;
        if self.check_seen(&loaded, context.loader, &context.active)? {
            return Ok(None);
        }
        context
            .budget
            .add(&loaded.id, &loaded.source, Span::default())
            .map_err(|error| vec![error])?;
        let (file, mut admission) = context
            .database
            .parse_with_route_admission(&loaded.id, &loaded.source)?;
        self.authorities
            .insert(loaded.id.clone(), context.loader.authority(&loaded.id));
        self.sources.insert(loaded.id.clone(), loaded.source);
        context.active.push(loaded.id.clone());
        let result = self.visit_imports(context, &file, &mut admission, implicit);
        context.active.pop();
        result.map(|()| Some(admission))
    }

    fn check_seen(
        &self,
        loaded: &LoadedSource,
        loader: &dyn SourceLoader,
        active: &[String],
    ) -> Result<bool, Vec<Diagnostic>> {
        let Some(source) = self.sources.get(&loaded.id) else {
            return Ok(false);
        };
        let importer = active.last().map_or(loaded.id.as_str(), String::as_str);
        if source != &loaded.source {
            return Err(vec![source_collision(importer, &loaded.id)]);
        }
        if self.authorities.get(&loaded.id) != Some(&loader.authority(&loaded.id)) {
            return Err(vec![authority_collision(importer, &loaded.id)]);
        }
        Ok(true)
    }

    fn visit_imports(
        &mut self,
        context: &mut VisitContext<'_>,
        file: &SurfaceFile,
        admission: &mut DependencyRouteAdmission,
        implicit: &[String],
    ) -> Result<(), Vec<Diagnostic>> {
        let mut routes = BTreeMap::new();
        for requested in implicit {
            self.visit_request(context, &file.path, requested, Span::default(), &mut routes)?;
        }
        for import in &file.imports {
            self.visit_request(context, &file.path, &import.path, import.span, &mut routes)?;
        }
        if context.database.commit_routes(admission, routes) == DependencyRouteCommit::RetryRequired
        {
            self.retry_required = true;
        }
        Ok(())
    }

    fn visit_request(
        &mut self,
        context: &mut VisitContext<'_>,
        importer: &str,
        requested: &str,
        span: Span,
        routes: &mut BTreeMap<String, String>,
    ) -> Result<(), Vec<Diagnostic>> {
        let route = (importer.to_owned(), requested.to_owned());
        if let Some(resolved) = self.resolutions.get(&route) {
            routes.insert(requested.to_owned(), resolved.clone());
            return Ok(());
        }
        let loaded = context
            .loader
            .load(importer, requested)
            .map_err(|message| {
                vec![Diagnostic::new(
                    "PROGRAM_IMPORT_LOAD",
                    importer,
                    message,
                    span,
                )]
            })?;
        validate_source_id(&loaded.id).map_err(|message| {
            vec![Diagnostic::new(
                "PROGRAM_SOURCE_ID",
                importer,
                message,
                span,
            )]
        })?;
        self.resolutions.insert(route, loaded.id.clone());
        routes.insert(requested.to_owned(), loaded.id.clone());
        self.visit(context, loaded, &[]).map(|_| ())
    }
}
