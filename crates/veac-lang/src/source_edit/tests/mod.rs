mod component_animation;
mod contract;
mod declaration_validation_branches;
mod error;
mod expression_path;
mod function;
mod implementation_target;
mod json;
mod method;
mod nominal_declaration;
mod revision;
mod statement;
mod strict_json_branches;
mod structural;
mod structural_errors;
mod target_identity_branches;
mod text_edit;
mod text_edit_limits;
mod validation;
mod validation_budget;
mod validation_errors;
mod validation_snapshot_branches;

use super::*;

fn revision(value: char) -> SourceRevision {
    SourceRevision {
        source_graph_sha256: value.to_string().repeat(64),
    }
}

fn target() -> SourceNodeRef {
    SourceNodeRef::constant("timeline/main.veac", "duration")
}

fn operation(source: &str) -> SourceEditOperation {
    SourceEditOperation::SetExpression {
        target: target(),
        site: ExpressionSite::ConstantValue,
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
