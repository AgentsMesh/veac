use clap::{ArgMatches, Command as ClapCommand};

use super::super::shared::{flag, flag_value, path, path_option, path_value, required_path_value};
use crate::arguments::Command;

pub(super) fn commands() -> [ClapCommand; 2] {
    [check_ir(), edit()]
}

fn check_ir() -> ClapCommand {
    ClapCommand::new("check-ir")
        .about("Validate strict canonical project JSON")
        .arg(path("project"))
}

fn edit() -> ClapCommand {
    ClapCommand::new("edit")
        .about("Apply one atomic typed edit batch to canonical project JSON")
        .arg(path("project"))
        .arg(path("edit_batch"))
        .arg(
            path_option("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("Write the accepted project to PATH; defaults to updating PROJECT in place"),
        )
        .arg(
            flag("dry_run", "dry-run")
                .help("Compute and emit the outcome without writing a project"),
        )
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "check-ir" => Command::CheckIr {
            project: required_path_value(matches, "project"),
        },
        "edit" => Command::Edit {
            project: required_path_value(matches, "project"),
            edit_batch: required_path_value(matches, "edit_batch"),
            output: path_value(matches, "output"),
            dry_run: flag_value(matches, "dry_run"),
        },
        _ => unreachable!("clap only accepts registered editing commands"),
    }
}
