use clap::{Arg, ArgAction, ArgMatches, Command as ClapCommand};

use crate::arguments::Command;

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("language-spec")
        .about("Print the versioned VEAC language contract")
        .arg(
            Arg::new("schema")
                .long("schema")
                .help("Print the language-spec JSON Schema")
                .action(ArgAction::SetTrue),
        )
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    Command::LanguageSpec {
        schema: matches.get_flag("schema"),
    }
}
