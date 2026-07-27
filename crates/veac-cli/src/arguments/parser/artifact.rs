use clap::{Arg, ArgAction, ArgMatches, Command as ClapCommand};

use super::shared::{
    path, path_option, path_value, required_path_value, required_string, string_value, value,
};
use crate::arguments::{ArtifactCommand, Command, PackageBindingsArgs, RelinkArgs};

pub(super) fn commands() -> [ClapCommand; 3] {
    [artifact(), package_bindings(), relink()]
}

fn artifact() -> ClapCommand {
    ClapCommand::new("artifact")
        .about("Inspect or remove a verified content-addressed artifact")
        .subcommand_required(true)
        .subcommand(
            ClapCommand::new("inspect")
                .arg(path("store"))
                .arg(value("key").required(true))
                .arg(path_option("output").short('o').long("output")),
        )
        .subcommand(
            ClapCommand::new("materialize")
                .arg(path("store"))
                .arg(value("key").required(true))
                .arg(path("destination")),
        )
        .subcommand(
            ClapCommand::new("remove")
                .arg(path("store"))
                .arg(value("key").required(true)),
        )
}

fn package_bindings() -> ClapCommand {
    ClapCommand::new("package-bindings")
        .about("Verify a portable package and emit machine-local input bindings")
        .arg(path("package"))
        .arg(path_option("output").short('o').long("output"))
}

fn relink() -> ClapCommand {
    ClapCommand::new("relink")
        .about("Discover identity-matched media and emit machine-local input bindings")
        .arg(path("project"))
        .arg(value("config").long("config"))
        .arg(
            Arg::new("search")
                .long("search")
                .required(true)
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(std::path::PathBuf)),
        )
        .arg(path_option("output").short('o').long("output"))
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "artifact" => Command::Artifact {
            command: artifact_matches(matches),
        },
        "package-bindings" => Command::PackageBindings(PackageBindingsArgs {
            package: required_path_value(matches, "package"),
            output: path_value(matches, "output"),
        }),
        "relink" => Command::Relink(RelinkArgs {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            search: matches
                .get_many::<std::path::PathBuf>("search")
                .expect("at least one search root was required")
                .cloned()
                .collect(),
            output: path_value(matches, "output"),
        }),
        _ => unreachable!("clap only accepts registered artifact commands"),
    }
}

fn artifact_matches(matches: &ArgMatches) -> ArtifactCommand {
    let (name, matches) = matches
        .subcommand()
        .expect("artifact subcommand is required");
    let store = required_path_value(matches, "store");
    let key = required_string(matches, "key");
    match name {
        "inspect" => ArtifactCommand::Inspect {
            store,
            key,
            output: path_value(matches, "output"),
        },
        "materialize" => ArtifactCommand::Materialize {
            store,
            key,
            destination: required_path_value(matches, "destination"),
        },
        "remove" => ArtifactCommand::Remove { store, key },
        _ => unreachable!("clap only accepts registered artifact subcommands"),
    }
}
