use crate::arguments::{ArtifactCommand, Cli, Command, SchemaContract};

fn parse(arguments: &[&str]) -> Command {
    Cli::try_parse_from(arguments).unwrap().command
}

#[test]
fn artifact_binding_and_relink_commands_have_typed_argument_shapes() {
    assert!(matches!(
        parse(&[
            "veac",
            "artifact",
            "inspect",
            "store",
            "abcd",
            "--output",
            "info.json"
        ]),
        Command::Artifact {
            command: ArtifactCommand::Inspect {
                output: Some(_),
                ..
            }
        }
    ));
    assert!(matches!(
        parse(&["veac", "artifact", "remove", "store", "abcd"]),
        Command::Artifact {
            command: ArtifactCommand::Remove { .. }
        }
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "artifact",
            "materialize",
            "store",
            "abcd",
            "payload.bin"
        ]),
        Command::Artifact {
            command: ArtifactCommand::Materialize { .. }
        }
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "package-bindings",
            "bundle",
            "--output",
            "bindings.json"
        ]),
        Command::PackageBindings(_)
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "relink",
            "project.json",
            "--search",
            "one",
            "--search",
            "two"
        ]),
        Command::Relink(arguments) if arguments.search.len() == 2
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "plan",
            "project.json",
            "--bindings",
            "bindings.json"
        ]),
        Command::Plan {
            bindings: Some(_),
            ..
        }
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "render",
            "project.json",
            "--bindings",
            "bindings.json"
        ]),
        Command::Render {
            bindings: Some(_),
            ..
        }
    ));
    assert!(matches!(
        parse(&["veac", "schema", "--contract", "execution-bindings"]),
        Command::Schema {
            contract: SchemaContract::ExecutionBindings,
            ..
        }
    ));
}

#[test]
fn artifact_commands_reject_incomplete_invocations() {
    assert!(Cli::try_parse_from(["veac", "artifact"]).is_err());
    assert!(Cli::try_parse_from(["veac", "artifact", "inspect", "store"]).is_err());
    assert!(Cli::try_parse_from(["veac", "artifact", "materialize", "store", "abcd"]).is_err());
    assert!(Cli::try_parse_from(["veac", "relink", "project.json"]).is_err());
}
