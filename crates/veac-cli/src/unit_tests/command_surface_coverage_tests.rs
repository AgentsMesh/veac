use std::path::PathBuf;

use tempfile::tempdir;

use super::support::{source_file, EXECUTABLE_SOURCE};
use crate::arguments::{
    ArtifactCommand, LanguagePackageCommand, PackageBindingsArgs, ProjectCommand,
};
use crate::{SchemaContract, SchemaFormat};

#[test]
fn source_graph_commands_execute_in_process() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE_SOURCE);

    crate::commands::source_index(&source, &[]).unwrap();
    crate::commands::source_revision(&source, &[]).unwrap();
}

#[test]
fn language_spec_and_package_search_execute_in_process() {
    let temp = tempdir().unwrap();
    let store = temp.path().join("store");
    std::fs::create_dir_all(store.join("plain-directory")).unwrap();

    crate::commands::language_spec(false).unwrap();
    crate::commands::language_spec(true).unwrap();
    crate::commands::language_package(LanguagePackageCommand::Search {
        store: store.clone(),
        query: "clip".to_owned(),
    })
    .unwrap();

    let error = crate::commands::language_package(LanguagePackageCommand::Search {
        store,
        query: "Clip Tools".to_owned(),
    })
    .unwrap_err();
    assert!(error.to_string().contains("LANG_PACKAGE_QUERY"));
}

#[test]
fn project_read_only_commands_execute_in_process() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project.veac");
    let source = include_str!("../../../veac-project/tests/fixtures/authored_minimal.veac")
        .replace("},\n    ],", "}\n    ],")
        .replace("},\n        ],", "}\n        ],");
    std::fs::write(&project, source).unwrap();
    let commands = [
        ProjectCommand::Check {
            project: project.clone(),
            package_roots: vec![],
        },
        ProjectCommand::Inspect {
            project: project.clone(),
            package_roots: vec![],
        },
        ProjectCommand::Graph {
            project,
            package_roots: vec![],
        },
    ];

    for command in commands {
        crate::commands::project(command).unwrap();
    }
}

#[test]
fn schema_and_artifact_errors_stay_machine_readable() {
    for contract in [
        SchemaContract::Project,
        SchemaContract::SourceIndex,
        SchemaContract::ExecutionBindings,
        SchemaContract::LanguageSpec,
    ] {
        crate::commands::schema(contract, SchemaFormat::JsonSchema).unwrap();
    }

    let temp = tempdir().unwrap();
    let store = temp.path().join("artifacts");
    std::fs::create_dir(&store).unwrap();
    let error = crate::commands::artifact(ArtifactCommand::Inspect {
        store,
        key: "invalid-digest".to_owned(),
        output: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("ARTIFACT"));
}

#[test]
fn package_bindings_rejects_non_package_roots_without_panicking() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("not-a-package");
    std::fs::write(&file, b"file").unwrap();
    let error = crate::commands::package_bindings(PackageBindingsArgs {
        package: file,
        output: None::<PathBuf>,
    })
    .unwrap_err();
    assert!(error.to_string().contains("PACKAGE_BINDINGS_FAILED"));
}
