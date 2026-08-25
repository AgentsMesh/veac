#![allow(dead_code)]

use std::path::PathBuf;

use veac_artifact::{ArtifactStore, ContentDigest};
use veac_build::*;
use veac_project::*;

mod backend;
mod sources;

pub use backend::*;
pub use sources::*;

pub struct Fixture {
    pub temp: tempfile::TempDir,
    pub source: PathBuf,
    pub material: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let material = temp.path().join("material");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&material).unwrap();
        std::fs::write(source.join("plate.veac"), PROJECT).unwrap();
        std::fs::write(source.join("plate-module.veac"), PROJECT_MODULE).unwrap();
        std::fs::write(source.join("evidence.veac"), EVIDENCE).unwrap();
        std::fs::write(source.join("evidence-helper.veac"), EVIDENCE_HELPER).unwrap();
        std::fs::write(material.join("logo.bin"), b"logo").unwrap();
        std::fs::write(material.join("facts.json"), b"{}").unwrap();
        Self {
            temp,
            source,
            material,
        }
    }

    pub fn adapter(&self) -> ProjectGraphAdapter {
        ProjectGraphAdapter::new(
            &self.source,
            &self.material,
            ProjectPackageSet::capture(&[]).unwrap(),
        )
        .unwrap()
    }

    pub fn runtime(&self, backend: TestBackend) -> ProjectBuildRuntime<TestBackend> {
        ProjectBuildRuntime::new(
            BuildLimits::new(3, ResourceClaim::new(3, 1024, 0)).unwrap(),
            ArtifactStore::new(self.temp.path().join("artifacts")),
            self.temp.path().join("leases"),
            self.temp.path().join("staging"),
            self.temp.path().join("delivery"),
            backend,
        )
        .unwrap()
    }
}

pub fn graph() -> ResolvedTargetGraph {
    let plate = instance(
        "plate",
        ProjectTargetEntry::Veac {
            source: ProjectPath::new("plate.veac"),
        },
        vec![
            veac_project::ResolvedInput {
                id: InputId::from("logo"),
                source: ResolvedInputSource::ProjectMaterial {
                    path: ProjectPath::new("logo.bin"),
                },
            },
            veac_project::ResolvedInput {
                id: InputId::from("facts"),
                source: ResolvedInputSource::AssetFact {
                    path: ProjectPath::new("facts.json"),
                    fact: FactId::from("dimensions"),
                },
            },
        ],
        "out/plate.bin",
    );
    let proxy = instance(
        "proxy",
        ProjectTargetEntry::MediaDerivation {
            operation: MediaDerivation::ProxyVideo {
                source: InputId::from("source"),
                source_stream: ProjectStreamSelection {
                    global_index: 0,
                    type_index: 0,
                },
                source_clock: ProjectSourceClock::Identity {
                    duration: ProjectRational::new(1, 1),
                },
                width: 32,
                height: 24,
                frame_rate: ProjectRational::new(25, 1),
                crf: 28,
            },
        },
        vec![artifact_input("source", "plate")],
        "out/proxy.bin",
    );
    let final_node = instance(
        "final",
        ProjectTargetEntry::Evidence {
            contract: ProjectPath::new("evidence.veac"),
        },
        vec![artifact_input("source", "proxy")],
        "out/final.bin",
    );
    ResolvedTargetGraph {
        version: RESOLVED_GRAPH_VERSION,
        manifest_digest: format!("sha256:{}", ContentDigest::sha256(b"manifest").value),
        instances: vec![final_node, plate, proxy],
        edges: vec![
            edge("plate", "proxy", Some("source")),
            edge("proxy", "final", Some("source")),
        ],
        build_order: ["plate", "proxy", "final"]
            .map(TargetInstanceId::from)
            .to_vec(),
    }
}

pub fn single_graph() -> ResolvedTargetGraph {
    let mut value = graph();
    value.instances.retain(|item| item.id.as_str() == "plate");
    value.edges.clear();
    value.build_order = vec![TargetInstanceId::from("plate")];
    value
}

fn instance(
    id: &str,
    entry: ProjectTargetEntry,
    inputs: Vec<veac_project::ResolvedInput>,
    destination: &str,
) -> TargetInstance {
    TargetInstance {
        id: TargetInstanceId::from(id),
        target: TargetId::from(id),
        profile: Some(ProfileId::from("preview")),
        locale: Some(LocaleId::from("zh-cn")),
        matrix: MatrixAssignment::new(),
        entry,
        inputs,
        outputs: vec![ProjectOutput::Media {
            id: OutputId::from("video"),
            media_type: MediaType::Video,
        }],
        deliveries: vec![ResolvedDelivery {
            id: DeliveryId::from("file"),
            output: OutputId::from("video"),
            kind: DeliveryKind::File,
            destination: ProjectPath::new(destination),
        }],
    }
}

fn artifact_input(id: &str, dependency: &str) -> veac_project::ResolvedInput {
    veac_project::ResolvedInput {
        id: InputId::from(id),
        source: ResolvedInputSource::Artifact {
            instances: vec![TargetInstanceId::from(dependency)],
            output: OutputId::from("video"),
        },
    }
}

fn edge(dependency: &str, consumer: &str, binding: Option<&str>) -> ResolvedTargetEdge {
    ResolvedTargetEdge {
        dependency: TargetInstanceId::from(dependency),
        consumer: TargetInstanceId::from(consumer),
        binding: binding.map(InputId::from),
        output: binding.map(|_| OutputId::from("video")),
    }
}
