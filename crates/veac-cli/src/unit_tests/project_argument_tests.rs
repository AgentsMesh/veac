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
                command: ProjectCommand::Check { project },
            }
            | Command::Project {
                command: ProjectCommand::Inspect { project },
            }
            | Command::Project {
                command: ProjectCommand::Graph { project },
            } => project,
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
        command: ProjectCommand::Build { project, receipt },
    } = command
    else {
        panic!("unexpected command")
    };
    assert_eq!(project, Path::new("project.veac"));
    assert_eq!(receipt.as_deref(), Some(Path::new("build/receipt.json")));
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
                command: ProjectCommand::Evidence { project, receipt },
            }
            | Command::Project {
                command: ProjectCommand::Test { project, receipt },
            } => (project, receipt),
            other => panic!("unexpected command: {other:?}"),
        };
        assert_eq!(project, Path::new("project.veac"));
        assert_eq!(receipt.as_deref(), Some(Path::new("build/receipt.json")));
    }
}

#[test]
fn project_command_requires_a_leaf_and_entry() {
    assert!(Cli::try_parse_from(["veac", "project"]).is_err());
    assert!(Cli::try_parse_from(["veac", "project", "check"]).is_err());
    assert!(Cli::try_parse_from(["veac", "project", "unknown", "project.veac"]).is_err());
}
