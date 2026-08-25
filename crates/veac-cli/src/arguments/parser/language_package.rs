use clap::{ArgMatches, Command as ClapCommand};

use super::shared::{path, required_path_value, required_string, value};
use crate::arguments::{Command, LanguagePackageCommand};

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("package")
        .about("Inspect exact, content-addressed VEAC language packages")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(operation(
            "api",
            "Print compiler-derived canonical API metadata",
        ))
        .subcommand(operation(
            "inspect",
            "Print a verified package discovery snapshot",
        ))
        .subcommand(
            ClapCommand::new("search")
                .about("Search one explicit local package store")
                .arg(path("store"))
                .arg(value("query").required(true)),
        )
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    let (name, matches) = matches
        .subcommand()
        .expect("package subcommand was required by clap");
    let command = match name {
        "api" => LanguagePackageCommand::Api {
            root: required_path_value(matches, "root"),
        },
        "inspect" => LanguagePackageCommand::Inspect {
            root: required_path_value(matches, "root"),
        },
        "search" => LanguagePackageCommand::Search {
            store: required_path_value(matches, "store"),
            query: required_string(matches, "query"),
        },
        _ => unreachable!("clap accepts only package operations"),
    };
    Command::LanguagePackage { command }
}

fn operation(name: &'static str, about: &'static str) -> ClapCommand {
    ClapCommand::new(name).about(about).arg(path("root"))
}
