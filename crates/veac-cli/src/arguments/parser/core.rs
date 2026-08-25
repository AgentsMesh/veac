use clap::{ArgMatches, Command as ClapCommand};

use crate::arguments::Command;

mod delivery;
mod editing;
mod language_spec;
mod schema;
mod source;

pub(super) fn commands() -> Vec<ClapCommand> {
    let mut commands = Vec::from(editing::commands());
    commands.extend(source::commands());
    commands.push(language_spec::command());
    commands.push(schema::command());
    commands.extend(delivery::commands());
    commands
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "build" | "check" | "fmt" | "source-revision" | "source-index" | "source-edit" => {
            source::from_matches(name, matches)
        }
        "check-ir" | "edit" => editing::from_matches(name, matches),
        "schema" => schema::from_matches(matches),
        "language-spec" => language_spec::from_matches(matches),
        _ => delivery::from_matches(name, matches),
    }
}
