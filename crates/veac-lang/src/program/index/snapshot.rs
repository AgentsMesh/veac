use crate::source_edit::{
    BodySite, DeclarationSite, ExpressionSite, SourceNodeRef, SourceSnapshot, StatementSite,
};

use super::SourceIndex;

impl SourceSnapshot for SourceIndex {
    fn node_exists(&self, target: &SourceNodeRef) -> bool {
        self.nodes.contains_key(target)
    }

    fn expression_source(&self, target: &SourceNodeRef, site: &ExpressionSite) -> Option<&str> {
        self.expression(target, site)
            .map(|value| value.source.as_str())
    }

    fn statement_source(&self, target: &SourceNodeRef, site: &StatementSite) -> Option<&str> {
        self.statement(target, site)
            .map(|value| value.source.as_str())
    }

    fn body_source(&self, target: &SourceNodeRef, site: BodySite) -> Option<&str> {
        self.body(target, site).map(|value| value.source.as_str())
    }

    fn declaration_source(&self, target: &SourceNodeRef, site: DeclarationSite) -> Option<&str> {
        self.declaration(target, site)
            .map(|value| value.source.as_str())
    }

    fn top_level_declaration_source(&self, target: &SourceNodeRef) -> Option<&str> {
        self.top_level(target).map(|value| value.source.as_str())
    }

    fn import_exists(&self, target: &crate::source_edit::SourceImportRef) -> bool {
        self.import(target).is_some()
    }

    fn import_path(&self, target: &crate::source_edit::SourceImportRef) -> Option<&str> {
        self.import(target).map(|value| value.path.as_str())
    }
}
