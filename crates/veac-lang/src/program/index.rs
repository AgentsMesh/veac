mod build;
mod component_animation;
mod expression;
mod fragment;
mod function;
mod inventory;
mod method;
mod nominal;
mod parameter_default;
mod snapshot;
mod storage;
mod structural;
mod temporal;
mod value;

pub use inventory::*;
pub use value::*;

use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::source_edit::{
    AuthoredSourceRevision, BodySite, DeclarationSite, ExpressionSite, SourceNodeRef,
    StatementSite, TextRange,
};

use super::diagnostic::Diagnostic;
use super::model::SurfaceFile;

#[derive(Debug, Clone)]
pub struct SourceIndex {
    revision: AuthoredSourceRevision,
    complete_revision: Option<super::PreparedSourceGraphRevision>,
    build_inputs: Vec<SourceIndexBuildInput>,
    expressions: BTreeMap<(SourceNodeRef, ExpressionSite), IndexedExpression>,
    statements: BTreeMap<(SourceNodeRef, StatementSite), IndexedStatement>,
    bodies: BTreeMap<(SourceNodeRef, BodySite), IndexedBody>,
    declarations: BTreeMap<(SourceNodeRef, DeclarationSite), IndexedDeclaration>,
    imports: BTreeMap<crate::source_edit::SourceImportRef, IndexedImport>,
    modules: BTreeMap<String, TextRange>,
    nodes: BTreeMap<SourceNodeRef, Span>,
    top_levels: BTreeMap<SourceNodeRef, IndexedTopLevelDeclaration>,
}

impl SourceIndex {
    pub fn revision(&self) -> &AuthoredSourceRevision {
        &self.revision
    }

    pub fn expression(
        &self,
        target: &SourceNodeRef,
        site: &ExpressionSite,
    ) -> Option<&IndexedExpression> {
        self.expressions.get(&(target.clone(), site.clone()))
    }

    pub fn statement(
        &self,
        target: &SourceNodeRef,
        site: &StatementSite,
    ) -> Option<&IndexedStatement> {
        self.statements.get(&(target.clone(), site.clone()))
    }

    pub fn body(&self, target: &SourceNodeRef, site: BodySite) -> Option<&IndexedBody> {
        self.bodies.get(&(target.clone(), site))
    }

    pub fn declaration(
        &self,
        target: &SourceNodeRef,
        site: DeclarationSite,
    ) -> Option<&IndexedDeclaration> {
        self.declarations.get(&(target.clone(), site))
    }

    fn file(&mut self, file: &SurfaceFile) -> Result<(), Diagnostic> {
        function::index(self, file)?;
        method::index(self, file)?;
        nominal::index(self, file)?;
        structural::index(self, file)?;
        temporal::index(self, file)?;
        for value in &file.constants {
            let target = SourceNodeRef::constant(&file.path, &value.name);
            self.register(&file.path, target.clone(), value.span)?;
            self.insert(
                &file.path,
                target,
                ExpressionSite::ConstantValue,
                file.syntax.slice_text(&value.expression),
                value.expression_span,
            )?;
        }
        for value in &file.inputs {
            let target = SourceNodeRef::input(&file.path, &value.name);
            self.register(&file.path, target.clone(), value.span)?;
            self.insert_declaration(
                &file.path,
                target,
                DeclarationSite::BuildInputDeclaration,
                file.source(),
                value.span,
            )?;
        }
        Ok(())
    }

    pub fn import(&self, target: &crate::source_edit::SourceImportRef) -> Option<&IndexedImport> {
        self.imports.get(target)
    }

    pub fn module_range(&self, module: &str) -> Option<TextRange> {
        self.modules.get(module).copied()
    }

    pub fn top_level(&self, target: &SourceNodeRef) -> Option<&IndexedTopLevelDeclaration> {
        self.top_levels.get(target)
    }

    fn register(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self.nodes.insert(target.clone(), span).is_some() {
            return Err(ambiguous(path, &target, span));
        }
        Ok(())
    }
}

pub(crate) use fragment::{
    validate_build_input_fragment, validate_import_fragment, validate_temporal_fragment,
    validate_top_level_fragment,
};

fn ambiguous(path: &str, target: &SourceNodeRef, span: Span) -> Diagnostic {
    Diagnostic::new(
        "SOURCE_INDEX_AMBIGUOUS_TARGET",
        path,
        format!("source target {:?} is ambiguous", target.path),
        span,
    )
}

#[cfg(test)]
#[path = "index/test_support.rs"]
mod test_support;
#[cfg(test)]
use test_support::revision as test_revision;

#[cfg(test)]
#[path = "index/build_input_inventory_tests.rs"]
mod build_input_inventory_tests;
#[cfg(test)]
#[path = "index/inventory_tests.rs"]
mod inventory_tests;
#[cfg(test)]
#[path = "index/revision_binding_tests.rs"]
mod revision_binding_tests;
#[cfg(test)]
#[path = "index/statement_tests.rs"]
mod statement_tests;
#[cfg(test)]
#[path = "index/structural_tests.rs"]
mod structural_tests;
