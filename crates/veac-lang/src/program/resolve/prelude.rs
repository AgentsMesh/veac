use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{Scope, SurfaceFile};

use super::Resolver;

impl Resolver<'_> {
    pub(super) fn entry_scope(
        &mut self,
        file: &SurfaceFile,
        preludes: &[String],
    ) -> Result<Scope, Diagnostic> {
        let mut scope = Scope::default();
        for requested in preludes {
            let imported = self.module(&file.path, requested, Span::default())?;
            super::names::prelude_types(
                &file.path,
                imported.as_ref(),
                &mut scope,
                Span::default(),
                &mut self.retained,
            )?;
        }
        self.scope_from(file, false, scope)
    }
}
