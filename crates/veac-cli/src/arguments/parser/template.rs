use clap::{ArgMatches, Command as ClapCommand};

use super::shared::{path, path_option, path_value, required_path_value};
use crate::arguments::{Command, TemplateCommand};

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("template")
        .about("Typed template-slot edit proposals")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            ClapCommand::new("propose")
                .about("Produce an atomic EditBatch that fills every typed slot")
                .arg(path("project"))
                .arg(path("request"))
                .arg(path_option("output").short('o').long("output")),
        )
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    let (name, matches) = matches
        .subcommand()
        .expect("template subcommand was required by clap");
    match name {
        "propose" => Command::Template {
            command: TemplateCommand::Propose {
                project: required_path_value(matches, "project"),
                request: required_path_value(matches, "request"),
                output: path_value(matches, "output"),
            },
        },
        _ => unreachable!("clap only accepts registered template commands"),
    }
}
