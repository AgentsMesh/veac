use std::sync::Arc;

use crate::authoring::Span;

use super::super::diagnostic::Diagnostic;
use super::super::loader::{validate_source_id, LoadedSource};
use super::super::model::{FileKind, Scope};
use super::Resolver;

impl Resolver<'_> {
    pub(super) fn module(
        &mut self,
        importer: &str,
        requested: &str,
        span: Span,
    ) -> Result<(Arc<Scope>, String), Diagnostic> {
        let loaded = self
            .loader
            .load(importer, requested)
            .map_err(|message| Diagnostic::new("PROGRAM_IMPORT_LOAD", importer, message, span))?;
        validate_source_id(&loaded.id)
            .map_err(|message| Diagnostic::new("PROGRAM_SOURCE_ID", importer, message, span))?;
        self.reject_source_collision(importer, &loaded, span)?;
        if let Some(scope) = self.cache.get(&loaded.id) {
            return Ok((Arc::clone(scope), loaded.id));
        }
        super::active::check(&self.active, importer, &loaded.id, span)?;
        self.budget.add(&loaded.id, &loaded.source, span)?;
        self.active.push(loaded.id.clone());
        self.sources
            .insert(loaded.id.clone(), loaded.source.clone());
        let result = self.load_module(importer, &loaded, span);
        self.active.pop();
        let scope = Arc::new(result?);
        let source_id = loaded.id.clone();
        self.cache.insert(source_id.clone(), Arc::clone(&scope));
        Ok((scope, source_id))
    }

    fn reject_source_collision(
        &self,
        importer: &str,
        loaded: &LoadedSource,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self
            .sources
            .get(&loaded.id)
            .is_some_and(|source| source != &loaded.source)
        {
            return Err(Diagnostic::new(
                "PROGRAM_SOURCE_ID_COLLISION",
                importer,
                format!("source ID `{}` resolved to different contents", loaded.id),
                span,
            ));
        }
        Ok(())
    }

    fn load_module(
        &mut self,
        importer: &str,
        loaded: &LoadedSource,
        span: Span,
    ) -> Result<Scope, Diagnostic> {
        let (file, admission) = self
            .database
            .reuse_parse_with_route_admission(&loaded.id, &loaded.source)
            .map_err(|errors| errors[0].clone())?;
        if !matches!(file.kind, FileKind::Module) {
            return Err(Diagnostic::new(
                "PROGRAM_IMPORT_PROJECT",
                importer,
                "imported file must contain a module",
                span,
            ));
        }
        self.route_admissions.insert(loaded.id.clone(), admission);
        self.scope(&file, true)
    }
}
