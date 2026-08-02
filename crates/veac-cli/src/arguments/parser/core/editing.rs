use clap::{Arg, ArgMatches, Command as ClapCommand};

use super::super::shared::{
    flag, flag_value, path, path_option, path_value, required_path_value, value,
};
use crate::arguments::Command;

pub(super) fn commands() -> [ClapCommand; 8] {
    [
        compile(),
        check(),
        format(),
        check_ir(),
        edit(),
        source_revision(),
        source_index(),
        source_edit(),
    ]
}

fn revision() -> Arg {
    value("revision")
        .long("revision")
        .value_parser(clap::value_parser!(u64))
        .default_value("0")
}

fn compile() -> ClapCommand {
    ClapCommand::new("compile")
        .about("Lower agent-oriented source into canonical project JSON")
        .arg(path("source"))
        .arg(
            path_option("emit_ir")
                .long("emit-ir")
                .value_name("PATH")
                .help("Write canonical IR to PATH; omit it (or use `-`) for stdout"),
        )
        .arg(revision())
}

fn check() -> ClapCommand {
    ClapCommand::new("check")
        .about("Validate agent-oriented source without media I/O")
        .arg(path("source"))
        .arg(revision())
}

fn format() -> ClapCommand {
    ClapCommand::new("fmt")
        .about("Canonically format agent-oriented source")
        .arg(path("source"))
        .arg(flag("check", "check").conflicts_with("stdout"))
        .arg(flag("stdout", "stdout"))
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

fn source_revision() -> ClapCommand {
    ClapCommand::new("source-revision")
        .about("Print the exact revision of a .veac source graph")
        .arg(path("source"))
}

fn source_index() -> ClapCommand {
    ClapCommand::new("source-index")
        .about("Print the stable agent-readable inventory of editable .veac source nodes")
        .arg(path("source"))
}

fn source_edit() -> ClapCommand {
    ClapCommand::new("source-edit")
        .about("Apply one atomic typed edit batch to .veac source of truth")
        .arg(path("source"))
        .arg(path("source_edit_batch"))
        .arg(
            path_option("output")
                .short('o')
                .long("output")
                .visible_alias("out")
                .value_name("PATH")
                .help("Write the edited module to PATH; defaults to updating it in place"),
        )
        .arg(flag("dry_run", "dry-run").help("Validate and report without writing source"))
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "compile" => Command::Compile {
            source: required_path_value(matches, "source"),
            emit_ir: path_value(matches, "emit_ir"),
            revision: revision_value(matches),
        },
        "check" => Command::Check {
            source: required_path_value(matches, "source"),
            revision: revision_value(matches),
        },
        "fmt" => Command::Fmt {
            source: required_path_value(matches, "source"),
            check: flag_value(matches, "check"),
            stdout: flag_value(matches, "stdout"),
        },
        "check-ir" => Command::CheckIr {
            project: required_path_value(matches, "project"),
        },
        "edit" => Command::Edit {
            project: required_path_value(matches, "project"),
            edit_batch: required_path_value(matches, "edit_batch"),
            output: path_value(matches, "output"),
            dry_run: flag_value(matches, "dry_run"),
        },
        "source-revision" => Command::SourceRevision {
            source: required_path_value(matches, "source"),
        },
        "source-index" => Command::SourceIndex {
            source: required_path_value(matches, "source"),
        },
        "source-edit" => Command::SourceEdit {
            source: required_path_value(matches, "source"),
            source_edit_batch: required_path_value(matches, "source_edit_batch"),
            output: path_value(matches, "output"),
            dry_run: flag_value(matches, "dry_run"),
        },
        _ => unreachable!("clap only accepts registered editing commands"),
    }
}

fn revision_value(matches: &ArgMatches) -> u64 {
    *matches
        .get_one::<u64>("revision")
        .expect("revision has a clap default")
}
