use clap::{ArgMatches, Command as ClapCommand};

use super::shared::{path, path_option, path_value, path_values, required_path_value};
use crate::arguments::{Command, ProjectCommand};

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("project")
        .about("Validate and inspect an authored VEAC project workspace")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            leaf(
                "check",
                "Validate the authored project and its resolved graph",
            ),
            leaf("inspect", "Print the canonical project manifest"),
            leaf("graph", "Print the canonical resolved target graph"),
            execution_leaf("build", "Build, cache, and publish all project targets"),
            execution_leaf(
                "evidence",
                "Build and publish typed evidence bundles without gating assertion failures",
            ),
            execution_leaf(
                "test",
                "Build evidence bundles and fail when an assertion fails or errors",
            ),
        ])
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    let (name, matches) = matches
        .subcommand()
        .expect("project subcommand was required by clap");
    let command = match name {
        "check" => ProjectCommand::Check {
            project: required_path_value(matches, "project"),
            package_roots: path_values(matches, "package_roots"),
        },
        "inspect" => ProjectCommand::Inspect {
            project: required_path_value(matches, "project"),
            package_roots: path_values(matches, "package_roots"),
        },
        "graph" => ProjectCommand::Graph {
            project: required_path_value(matches, "project"),
            package_roots: path_values(matches, "package_roots"),
        },
        "build" => ProjectCommand::Build {
            project: required_path_value(matches, "project"),
            receipt: path_value(matches, "receipt"),
            package_roots: path_values(matches, "package_roots"),
        },
        "evidence" => ProjectCommand::Evidence {
            project: required_path_value(matches, "project"),
            receipt: path_value(matches, "receipt"),
            package_roots: path_values(matches, "package_roots"),
        },
        "test" => ProjectCommand::Test {
            project: required_path_value(matches, "project"),
            receipt: path_value(matches, "receipt"),
            package_roots: path_values(matches, "package_roots"),
        },
        _ => unreachable!("project command was validated by clap"),
    };
    Command::Project { command }
}

fn leaf(name: &'static str, about: &'static str) -> ClapCommand {
    ClapCommand::new(name)
        .about(about)
        .arg(path("project"))
        .arg(package_root())
}

fn package_root() -> clap::Arg {
    path_option("package_roots")
        .long("package-root")
        .action(clap::ArgAction::Append)
        .help("Mount one exact verified package root; repeat to mount more")
}

fn execution_leaf(name: &'static str, about: &'static str) -> ClapCommand {
    leaf(name, about).arg(
        path_option("receipt")
            .long("receipt")
            .value_name("PATH")
            .help("Write the canonical build receipt to PATH instead of stdout"),
    )
}
