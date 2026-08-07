use super::cache;
use crate::{PlanSource, ResolverFingerprint};
use veac_ir::{
    ExecutableDigests, ExecutableManifest, ProjectId, TemporalProgramLibrary,
    TEMPORAL_OPSET_VERSION,
};

fn source(compiler_sha256: &str) -> PlanSource {
    PlanSource {
        project_id: ProjectId::new("prj_identity").unwrap(),
        revision: 1,
        timebase: 600,
        semantic_hash: "a".repeat(64),
        snapshot_hash: "b".repeat(64),
        executable: ExecutableManifest::current(
            "0.1.0",
            ExecutableDigests {
                domain_registry_sha256: "c".repeat(64),
                main_core_sha256: "d".repeat(64),
                source_graph_sha256: "e".repeat(64),
                declared_inputs_sha256: "f".repeat(64),
                compiler_sha256: compiler_sha256.to_owned(),
            },
        ),
    }
}

#[test]
fn compiler_build_identity_is_part_of_plan_cache_identity() {
    let resolver = ResolverFingerprint {
        resolver_version: "resolver".to_owned(),
        stream_selection_policy: "policy".to_owned(),
        effect_registry_version: "effects".to_owned(),
        capability_profile: "profile".to_owned(),
    };
    let temporal = TemporalProgramLibrary {
        opset_version: TEMPORAL_OPSET_VERSION,
        programs: Vec::new(),
        bindings: Vec::new(),
        provenance: Vec::new(),
    };
    let first = cache(&source(&"1".repeat(64)), &resolver, &temporal).unwrap();
    let second = cache(&source(&"2".repeat(64)), &resolver, &temporal).unwrap();
    assert_ne!(
        first.executable_manifest_sha256,
        second.executable_manifest_sha256
    );
    assert_eq!(
        first.project_semantic_sha256,
        second.project_semantic_sha256
    );
    assert_eq!(
        first.temporal_library_sha256,
        second.temporal_library_sha256
    );
}
