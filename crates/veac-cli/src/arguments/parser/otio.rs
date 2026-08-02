use clap::{ArgMatches, Command as ClapCommand};

use super::shared::{
    flag, flag_value, path, path_option, path_value, required_path_value, required_string,
    string_value, value,
};
use crate::arguments::{Command, OtioCommand};

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("otio")
        .about("Loss-aware OpenTimelineIO interchange and canonical edit proposals")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([export(), propose()])
}

fn lossy(command: ClapCommand) -> ClapCommand {
    command
        .arg(
            path_option("loss_report")
                .long("loss-report")
                .requires("allow_lossy"),
        )
        .arg(flag("allow_lossy", "allow-lossy").requires("loss_report"))
}

fn export() -> ClapCommand {
    lossy(
        ClapCommand::new("export")
            .about("Export one sequence as standard OTIO plus the lossless VEAC extension")
            .arg(path("project"))
            .arg(value("sequence").long("sequence"))
            .arg(path_option("output").short('o').long("output")),
    )
}

fn propose() -> ClapCommand {
    lossy(
        ClapCommand::new("propose")
            .about("Convert OTIO into a revision-checked, atomic canonical edit proposal")
            .arg(path("project"))
            .arg(path("timeline"))
            .arg(
                path_option("bindings")
                    .long("bindings")
                    .help("Required for third-party OTIO without a VEAC metadata extension"),
            )
            .arg(value("operation_id").long("operation-id").required(true))
            .arg(path_option("output").short('o').long("output")),
    )
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    let (name, matches) = matches
        .subcommand()
        .expect("OTIO subcommand was required by clap");
    Command::Otio {
        command: match name {
            "export" => OtioCommand::Export {
                project: required_path_value(matches, "project"),
                sequence: string_value(matches, "sequence"),
                output: path_value(matches, "output"),
                loss_report: path_value(matches, "loss_report"),
                allow_lossy: flag_value(matches, "allow_lossy"),
            },
            "propose" => OtioCommand::Propose {
                project: required_path_value(matches, "project"),
                timeline: required_path_value(matches, "timeline"),
                bindings: path_value(matches, "bindings"),
                operation_id: required_string(matches, "operation_id"),
                output: path_value(matches, "output"),
                loss_report: path_value(matches, "loss_report"),
                allow_lossy: flag_value(matches, "allow_lossy"),
            },
            _ => unreachable!("clap only accepts registered OTIO commands"),
        },
    }
}
