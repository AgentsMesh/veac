use clap::{Arg, ArgMatches, Command as ClapCommand};

use crate::arguments::Command;

mod artifact;
mod caption;
mod core;
mod otio;
mod shared;
mod template;
mod workflow;

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("veac")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Video Editing as Code")
        .arg(
            Arg::new("diagnostic_format")
                .long("diagnostic-format")
                .value_name("human|json")
                .value_parser(["human", "json"])
                .default_value("human")
                .global(true),
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(caption::command())
        .subcommand(template::command())
        .subcommands(artifact::commands())
        .subcommand(otio::command())
        .subcommands(workflow::commands())
        .subcommands(core::commands())
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    let (name, matches) = matches
        .subcommand()
        .expect("top-level subcommand was required by clap");
    match name {
        "caption" => caption::from_matches(matches),
        "template" => template::from_matches(matches),
        "artifact" | "package-bindings" | "relink" => artifact::from_matches(name, matches),
        "otio" => otio::from_matches(matches),
        "derive" | "provider-run" | "provider-propose" => workflow::from_matches(name, matches),
        _ => core::from_matches(name, matches),
    }
}
