use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgMatches, Command as ClapCommand};

use super::shared::{
    flag, flag_value, path, path_option, path_value, required_path_value, required_string, value,
};
use crate::arguments::{CaptionCommand, CaptionFormatArg, CaptionOverlapArg, Command};

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("caption")
        .about("Loss-aware caption sidecar interchange and canonical edit proposals")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([import(), export(), propose(), extract()])
}

fn format_arg() -> Arg {
    value("format")
        .long("format")
        .required(true)
        .value_parser(PossibleValuesParser::new(["srt", "web-vtt", "ass"]))
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

fn import() -> ClapCommand {
    lossy(
        ClapCommand::new("import")
            .about("Convert an SRT, WebVTT, or ASS sidecar to a canonical caption document")
            .arg(path("input"))
            .arg(format_arg())
            .arg(
                value("timescale")
                    .long("timescale")
                    .value_parser(clap::value_parser!(u32))
                    .default_value("1000"),
            )
            .arg(
                value("overlap")
                    .long("overlap")
                    .value_parser(PossibleValuesParser::new(["reject", "allow"]))
                    .default_value("reject"),
            )
            .arg(
                value("namespace")
                    .long("namespace")
                    .default_value("caption"),
            )
            .arg(path_option("output").short('o').long("output")),
    )
}

fn export() -> ClapCommand {
    lossy(
        ClapCommand::new("export")
            .about("Convert a canonical caption document to an SRT, WebVTT, or ASS sidecar")
            .arg(path("document"))
            .arg(format_arg())
            .arg(path_option("output").short('o').long("output")),
    )
}

fn propose() -> ClapCommand {
    ClapCommand::new("propose")
        .about("Produce a checked canonical EditBatch that inserts one caption track")
        .arg(path("project"))
        .arg(path("document"))
        .arg(path("bindings"))
        .arg(value("operation_id").long("operation-id").required(true))
        .arg(path_option("output").short('o').long("output"))
}

fn extract() -> ClapCommand {
    ClapCommand::new("extract")
        .about("Extract one caption track into a canonical caption document")
        .arg(path("project"))
        .arg(value("track").long("track").required(true))
        .arg(path("bindings"))
        .arg(path_option("output").short('o').long("output"))
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    let (name, matches) = matches
        .subcommand()
        .expect("caption subcommand was required by clap");
    Command::Caption {
        command: match name {
            "import" => import_matches(matches),
            "export" => export_matches(matches),
            "propose" => CaptionCommand::Propose {
                project: required_path_value(matches, "project"),
                document: required_path_value(matches, "document"),
                bindings: required_path_value(matches, "bindings"),
                operation_id: required_string(matches, "operation_id"),
                output: path_value(matches, "output"),
            },
            "extract" => CaptionCommand::Extract {
                project: required_path_value(matches, "project"),
                track: required_string(matches, "track"),
                bindings: required_path_value(matches, "bindings"),
                output: path_value(matches, "output"),
            },
            _ => unreachable!("clap only accepts registered caption commands"),
        },
    }
}

fn import_matches(matches: &ArgMatches) -> CaptionCommand {
    CaptionCommand::Import {
        input: required_path_value(matches, "input"),
        format: caption_format(matches),
        timescale: *matches
            .get_one::<u32>("timescale")
            .expect("timescale has a clap default"),
        overlap: match required_string(matches, "overlap").as_str() {
            "allow" => CaptionOverlapArg::Allow,
            _ => CaptionOverlapArg::Reject,
        },
        namespace: required_string(matches, "namespace"),
        output: path_value(matches, "output"),
        loss_report: path_value(matches, "loss_report"),
        allow_lossy: flag_value(matches, "allow_lossy"),
    }
}

fn export_matches(matches: &ArgMatches) -> CaptionCommand {
    CaptionCommand::Export {
        document: required_path_value(matches, "document"),
        format: caption_format(matches),
        output: path_value(matches, "output"),
        loss_report: path_value(matches, "loss_report"),
        allow_lossy: flag_value(matches, "allow_lossy"),
    }
}

fn caption_format(matches: &ArgMatches) -> CaptionFormatArg {
    match required_string(matches, "format").as_str() {
        "srt" => CaptionFormatArg::Srt,
        "web-vtt" => CaptionFormatArg::WebVtt,
        "ass" => CaptionFormatArg::Ass,
        _ => unreachable!("caption format was validated by clap"),
    }
}
