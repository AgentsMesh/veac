use std::path::Path;

use crate::arguments::{Cli, Command, ProjectCommand};

fn parse(args: &[&str]) -> Command {
    Cli::try_parse_from(args).unwrap().command
}

#[test]
fn project_commands_parse_an_authored_entry_path() {
    for (name, expected) in [
        ("check", "project.veac"),
        ("inspect", "workspace/project.veac"),
        ("graph", "nested/project.veac"),
    ] {
        let command = parse(&["veac", "project", name, expected]);
        let project = match command {
            Command::Project {
                command:
                    ProjectCommand::Check {
                        project,
                        package_roots,
                    },
            }
            | Command::Project {
                command:
                    ProjectCommand::Inspect {
                        project,
                        package_roots,
                    },
            }
            | Command::Project {
                command:
                    ProjectCommand::Graph {
                        project,
                        package_roots,
                    },
            } => {
                assert!(package_roots.is_empty());
                project
            }
            other => panic!("unexpected command: {other:?}"),
        };
        assert_eq!(project, Path::new(expected));
    }
}

#[test]
fn project_build_parses_an_optional_receipt_path() {
    let command = parse(&[
        "veac",
        "project",
        "build",
        "project.veac",
        "--receipt",
        "build/receipt.json",
    ]);
    let Command::Project {
        command:
            ProjectCommand::Build {
                project,
                receipt,
                package_roots,
            },
    } = command
    else {
        panic!("unexpected command")
    };
    assert_eq!(project, Path::new("project.veac"));
    assert_eq!(receipt.as_deref(), Some(Path::new("build/receipt.json")));
    assert!(package_roots.is_empty());
}

#[test]
fn project_evidence_and_test_share_the_receipt_contract() {
    for name in ["evidence", "test"] {
        let command = parse(&[
            "veac",
            "project",
            name,
            "project.veac",
            "--receipt",
            "build/receipt.json",
        ]);
        let (project, receipt) = match command {
            Command::Project {
                command:
                    ProjectCommand::Evidence {
                        project,
                        receipt,
                        package_roots,
                    },
            }
            | Command::Project {
                command:
                    ProjectCommand::Test {
                        project,
                        receipt,
                        package_roots,
                    },
            } => {
                assert!(package_roots.is_empty());
                (project, receipt)
            }
            other => panic!("unexpected command: {other:?}"),
        };
        assert_eq!(project, Path::new("project.veac"));
        assert_eq!(receipt.as_deref(), Some(Path::new("build/receipt.json")));
    }
}

#[test]
fn every_project_command_preserves_explicit_package_roots() {
    for name in ["check", "inspect", "graph", "build", "evidence", "test"] {
        let command = parse(&[
            "veac",
            "project",
            name,
            "project.veac",
            "--package-root",
            "z-package",
            "--package-root",
            "a-package",
        ]);
        let roots = match command {
            Command::Project { command } => match command {
                ProjectCommand::Check { package_roots, .. }
                | ProjectCommand::Inspect { package_roots, .. }
                | ProjectCommand::Graph { package_roots, .. }
                | ProjectCommand::Build { package_roots, .. }
                | ProjectCommand::Evidence { package_roots, .. }
                | ProjectCommand::Test { package_roots, .. } => package_roots,
            },
            other => panic!("unexpected command: {other:?}"),
        };
        assert_eq!(roots, [Path::new("z-package"), Path::new("a-package")]);
    }
}

#[test]
fn project_command_requires_a_leaf_and_entry() {
    assert!(Cli::try_parse_from(["veac", "project"]).is_err());
    assert!(Cli::try_parse_from(["veac", "project", "check"]).is_err());
    assert!(Cli::try_parse_from(["veac", "project", "unknown", "project.veac"]).is_err());
}
