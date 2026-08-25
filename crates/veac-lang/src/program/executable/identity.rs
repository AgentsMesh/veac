use crate::program::expression::{CompiledFunction, ProgramIdentity};
use crate::program::PreparedSourceGraph;

pub(super) fn read(
    main: &CompiledFunction,
    source_graph: &PreparedSourceGraph,
    declared_inputs_sha256: String,
) -> ProgramIdentity {
    ProgramIdentity {
        core_version: main.body().version(),
        domain_opset: main.body().domain_opset().raw(),
        domain_registry_sha256: main.body().domain_registry_digest().to_string(),
        main_content_sha256: main.content_digest().to_string(),
        source_graph_sha256: source_graph.complete_revision().sha256().to_owned(),
        declared_inputs_sha256,
    }
}
