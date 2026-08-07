use veac_ir::*;

pub(super) fn project_envelope(project: Project) -> ProjectEnvelope {
    ProjectEnvelope::new(
        project,
        ExecutableManifest::current(
            "0.1.0",
            ExecutableDigests {
                domain_registry_sha256: "a".repeat(64),
                main_core_sha256: "b".repeat(64),
                source_graph_sha256: "c".repeat(64),
                declared_inputs_sha256: "d".repeat(64),
                compiler_sha256: "e".repeat(64),
            },
        ),
        TemporalProgramLibrary {
            opset_version: TEMPORAL_OPSET_VERSION,
            programs: vec![],
            bindings: vec![],
            provenance: vec![],
        },
    )
}
