mod contract;
mod definition;
mod definition_identity;
mod error;
mod json;
mod revision;
mod text_edit;
mod text_edit_limits;
mod validation;
mod validation_budget;
mod validation_errors;

use super::*;

fn revision(value: char) -> SourceRevision {
    SourceRevision {
        source_graph_sha256: value.to_string().repeat(64),
    }
}

fn target() -> SourceNodeRef {
    SourceNodeRef::item("timeline/main.veac", "timeline", "main", "visual", "hero")
}

fn operation(source: &str) -> SourceEditOperation {
    SourceEditOperation::SetExpression {
        target: target(),
        site: ExpressionSite::ItemRecordDuration,
        expression: ExpressionSource {
            source: source.to_owned(),
        },
    }
}

fn batch() -> SourceEditBatch {
    let mut value = SourceEditBatch::new(
        veac_ir::OperationId::new("op_source_test").unwrap(),
        revision('a'),
    );
    value.operations.push(operation("6s"));
    value
}
