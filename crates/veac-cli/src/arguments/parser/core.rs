use clap::{ArgMatches, Command as ClapCommand};

use crate::arguments::Command;

mod delivery;
mod editing;
mod schema;

pub(super) fn commands() -> Vec<ClapCommand> {
    let mut commands = Vec::from(editing::commands());
    commands.push(schema::command());
    commands.extend(delivery::commands());
    commands
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "compile" | "check" | "fmt" | "check-ir" | "edit" | "source-revision" | "source-index"
        | "source-edit" => editing::from_matches(name, matches),
        "schema" => schema::from_matches(matches),
        _ => delivery::from_matches(name, matches),
    }
}
