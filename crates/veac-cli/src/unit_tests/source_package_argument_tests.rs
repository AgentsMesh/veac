use std::path::PathBuf;

use crate::arguments::{Cli, Command};

#[test]
fn every_source_command_preserves_repeated_package_root_order() {
    for command in [
        vec!["build", "main.veac"],
        vec!["check", "main.veac"],
        vec!["fmt", "main.veac"],
        vec!["source-revision", "main.veac"],
        vec!["source-index", "main.veac"],
        vec!["source-edit", "main.veac", "edit.json"],
    ] {
        let mut arguments = vec!["veac"];
        arguments.extend(command);
        arguments.extend(["--package-root", "z-package", "--package-root", "a-package"]);
        let parsed = Cli::try_parse_from(arguments).unwrap();
        assert_eq!(roots(parsed.command), expected());
    }
}

fn roots(command: Command) -> Vec<PathBuf> {
    match command {
        Command::Build(value) => value.package_roots,
        Command::Check(value) => value.package_roots,
        Command::Fmt(value) => value.package_roots,
        Command::SourceRevision(value) | Command::SourceIndex(value) => value.package_roots,
        Command::SourceEdit(value) => value.package_roots,
        _ => panic!("fixture must parse a source command"),
    }
}

fn expected() -> Vec<PathBuf> {
    ["z-package", "a-package"].map(PathBuf::from).to_vec()
}
