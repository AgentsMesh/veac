use clap::{Arg, ArgMatches, Command as ClapCommand};

use super::super::shared::{
    flag, flag_value, material_root, path, path_option, path_value, required_path_value,
    string_values, value,
};
use crate::arguments::Command;

pub(super) fn commands() -> [ClapCommand; 8] {
    [
        build(),
        check(),
        format(),
        check_ir(),
        edit(),
        source_revision(),
        source_index(),
        source_edit(),
    ]
}

fn build() -> ClapCommand {
    ClapCommand::new("build")
        .about("Execute programmable VEAC source into canonical project JSON")
        .arg(path("source"))
        .arg(
            path_option("emit_ir")
                .long("emit-ir")
                .value_name("PATH")
                .help("Write IR to PATH; detached local-material IR requires --material-root"),
        )
        .arg(inputs())
        .arg(inline_input())
        .arg(material_root())
        .arg(revision())
}

fn inputs() -> Arg {
    path_option("inputs")
        .long("inputs")
        .value_name("PATH")
        .help("Bind declared Build inputs from a veac.build-inputs/v1 JSON manifest")
}

fn inline_input() -> Arg {
    value("inline_inputs")
        .long("input")
        .action(clap::ArgAction::Append)
        .help("Bind one declared Build input as NAME=VALUE; repeat to bind more")
}

fn revision() -> Arg {
    value("revision")
        .long("revision")
        .value_parser(clap::value_parser!(u64))
        .default_value("0")
}

fn check() -> ClapCommand {
    ClapCommand::new("check")
        .about("Validate executable VEAC source without media I/O")
        .arg(path("source"))
        .arg(inputs())
        .arg(inline_input())
        .arg(revision())
}

fn format() -> ClapCommand {
    ClapCommand::new("fmt")
        .about("Canonically format syntax-aware executable VEAC source")
        .arg(path("source"))
        .arg(
            flag("check", "check")
                .help("Fail when SOURCE is not canonically formatted")
                .conflicts_with("stdout"),
        )
        .arg(flag("stdout", "stdout").help("Print formatted source without writing SOURCE"))
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
        .arg(inputs())
        .arg(inline_input())
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
        "build" => Command::Build {
            source: required_path_value(matches, "source"),
            emit_ir: path_value(matches, "emit_ir"),
            inputs: path_value(matches, "inputs"),
            inline_inputs: string_values(matches, "inline_inputs"),
            material_root: path_value(matches, "material_root"),
            revision: revision_value(matches),
        },
        "check" => Command::Check {
            source: required_path_value(matches, "source"),
            inputs: path_value(matches, "inputs"),
            inline_inputs: string_values(matches, "inline_inputs"),
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
            inputs: path_value(matches, "inputs"),
            inline_inputs: string_values(matches, "inline_inputs"),
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
