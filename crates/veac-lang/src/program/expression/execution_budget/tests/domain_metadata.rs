use crate::program::expression::runtime::domain_graph::DomainGraphTransaction;
use crate::program::expression::{
    compile_functions, DomainOrigin, ExecutionDefinition, ExecutionFrame, ExpressionContext,
    FunctionDefinition, FunctionOrigin, PrimitiveType, ProgramIdentity, Value, ValueType,
};
use crate::program::{DomainOperationId, DomainOperationRegistry};

use super::{budget, unlimited, used, ExecutionBudget, Resource};

#[path = "domain_metadata/support.rs"]
mod support;
use support::*;

#[test]
fn authored_track_metadata_reserves_exact_bytes_before_attachment() {
    let registry = DomainOperationRegistry::standard();
    let origin = origin();
    let measured = ExecutionBudget::with_resource_limits(unlimited());
    let Attempt {
        graph,
        result,
        before: _,
    } = attach_layer(&registry, &measured, &origin);
    result.unwrap();
    let records = graph.record_count();
    let bytes = used(&measured, Resource::EmittedBytes);

    let exact = budget(Resource::EmittedBytes, bytes);
    let Attempt {
        graph,
        result,
        before: _,
    } = attach_layer(&registry, &exact, &origin);
    result.unwrap();
    assert_eq!(graph.record_count(), records);
    assert_eq!(used(&exact, Resource::EmittedBytes), bytes);

    let short = budget(Resource::EmittedBytes, bytes - 1);
    let Attempt {
        mut graph,
        result,
        before,
    } = attach_layer(&registry, &short, &origin);
    assert_eq!(result.unwrap_err().code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(graph.record_count(), before.records);
    assert_eq!(used(&short, Resource::EmittedBytes), before.bytes);
    assert_eq!(used(&short, Resource::EmittedEntities), before.entities);
    let error = graph
        .evaluate(DomainOperationId::Canvas.opcode(), vec![], 20..21)
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_TRANSACTION_FAILED");
}

#[test]
fn program_identity_reservation_is_exact_and_atomic() {
    let registry = DomainOperationRegistry::standard();
    let identity = identity();
    let bytes = identity.logical_bytes();

    let exact = budget(Resource::EmittedBytes, bytes);
    let mut graph = DomainGraphTransaction::new(&registry, &exact);
    graph.set_program_identity(identity.clone(), 1..2).unwrap();
    assert_eq!(used(&exact, Resource::EmittedBytes), bytes);

    let short = budget(Resource::EmittedBytes, bytes - 1);
    let mut graph = DomainGraphTransaction::new(&registry, &short);
    let error = graph.set_program_identity(identity, 3..4).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(used(&short, Resource::EmittedBytes), 0);
    assert_eq!(
        graph.host_context(5..6).unwrap_err().code(),
        "DOMAIN_TRANSACTION_FAILED"
    );
}

fn origin() -> DomainOrigin {
    let function = FunctionDefinition::new(
        "build",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        "{ 1 }",
    )
    .with_origin(FunctionOrigin::new("main.veac", 100..200));
    let functions = compile_functions(&ExpressionContext::empty(), &[function]).unwrap();
    let function = functions.functions().lookup("build").unwrap();
    let definition = ExecutionDefinition::function(function).unwrap();
    DomainOrigin::capture(&[ExecutionFrame::new(definition, None)], &[], 10..11).unwrap()
}

fn identity() -> ProgramIdentity {
    ProgramIdentity {
        core_version: 7,
        domain_opset: 2,
        domain_registry_sha256: "a".repeat(64),
        main_content_sha256: "b".repeat(64),
        source_graph_sha256: "c".repeat(64),
        declared_inputs_sha256: "d".repeat(64),
    }
}

fn identifier(value: &str) -> Value {
    Value::Identifier(value.into())
}
