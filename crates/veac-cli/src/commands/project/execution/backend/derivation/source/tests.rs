use std::collections::BTreeMap;

use super::bind;
use veac_build::ProjectComputation;
use veac_project::{
    InputId, ProjectLiteral, ResolvedInput, ResolvedInputSource, TargetId, TargetInstanceId,
};

fn computation(inputs: Vec<ResolvedInput>) -> ProjectComputation {
    ProjectComputation {
        instance: TargetInstanceId::from("derive"),
        target: TargetId::from("derive"),
        profile: None,
        locale: None,
        matrix: BTreeMap::new(),
        inputs,
        outputs: Vec::new(),
        bound_sources: Vec::new(),
        package_mounts: Vec::new(),
    }
}

#[test]
fn derivation_source_requires_a_bound_media_authority() {
    let source = InputId::from("source");
    let empty = computation(Vec::new());
    let temp = tempfile::tempdir().unwrap();
    assert!(bind(&empty, &source, &[], temp.path())
        .err()
        .unwrap()
        .to_string()
        .contains("is missing"));

    let literal = computation(vec![ResolvedInput {
        id: source.clone(),
        source: ResolvedInputSource::Literal {
            value: ProjectLiteral::Bool { value: true },
        },
    }]);
    assert!(bind(&literal, &source, &[], temp.path())
        .err()
        .unwrap()
        .to_string()
        .contains("project material or one artifact"));
}
