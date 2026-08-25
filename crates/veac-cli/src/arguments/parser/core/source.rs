use clap::{Arg, ArgMatches, Command as ClapCommand};

use super::super::shared::{
    flag, flag_value, material_root, path, path_option, path_value, path_values,
    required_path_value, string_values, value,
};
use crate::arguments::{
    BuildSourceArgs, CheckSourceArgs, Command, FormatSourceArgs, SourceEditArgs, SourceGraphArgs,
};

pub(super) fn commands() -> [ClapCommand; 6] {
    [
        build(),
        check(),
        format(),
        source_revision(),
        source_index(),
        source_edit(),
    ]
}

fn package_root() -> Arg {
    path_option("package_roots")
        .long("package-root")
        .action(clap::ArgAction::Append)
        .help("Mount one exact, verified VEAC package root; repeat for multiple packages")
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
        .arg(package_root())
        .arg(revision())
}

fn check() -> ClapCommand {
    ClapCommand::new("check")
        .about("Validate executable VEAC source without media I/O")
        .arg(path("source"))
        .arg(inputs())
        .arg(inline_input())
        .arg(package_root())
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
        .arg(package_root())
}

fn source_revision() -> ClapCommand {
    ClapCommand::new("source-revision")
        .about("Print the exact revision of a .veac source graph")
        .arg(path("source"))
        .arg(package_root())
}

fn source_index() -> ClapCommand {
    ClapCommand::new("source-index")
        .about("Print the stable agent-readable inventory of editable .veac source nodes")
        .arg(path("source"))
        .arg(package_root())
}

fn source_edit() -> ClapCommand {
    ClapCommand::new("source-edit")
        .about("Apply one atomic typed edit batch to .veac source of truth")
        .arg(path("source"))
        .arg(path("source_edit_batch"))
        .arg(inputs())
        .arg(inline_input())
        .arg(package_root())
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
    let source = || required_path_value(matches, "source");
    let packages = || path_values(matches, "package_roots");
    match name {
        "build" => Command::Build(BuildSourceArgs {
            source: source(),
            emit_ir: path_value(matches, "emit_ir"),
            inputs: path_value(matches, "inputs"),
            inline_inputs: string_values(matches, "inline_inputs"),
            material_root: path_value(matches, "material_root"),
            package_roots: packages(),
            revision: revision_value(matches),
        }),
        "check" => Command::Check(CheckSourceArgs {
            source: source(),
            inputs: path_value(matches, "inputs"),
            inline_inputs: string_values(matches, "inline_inputs"),
            package_roots: packages(),
            revision: revision_value(matches),
        }),
        "fmt" => Command::Fmt(FormatSourceArgs {
            source: source(),
            check: flag_value(matches, "check"),
            stdout: flag_value(matches, "stdout"),
            package_roots: packages(),
        }),
        "source-revision" => Command::SourceRevision(SourceGraphArgs {
            source: source(),
            package_roots: packages(),
        }),
        "source-index" => Command::SourceIndex(SourceGraphArgs {
            source: source(),
            package_roots: packages(),
        }),
        "source-edit" => Command::SourceEdit(SourceEditArgs {
            source: source(),
            source_edit_batch: required_path_value(matches, "source_edit_batch"),
            inputs: path_value(matches, "inputs"),
            inline_inputs: string_values(matches, "inline_inputs"),
            package_roots: packages(),
            output: path_value(matches, "output"),
            dry_run: flag_value(matches, "dry_run"),
        }),
        _ => unreachable!("clap only accepts registered source commands"),
    }
}

fn revision_value(matches: &ArgMatches) -> u64 {
    *matches
        .get_one::<u64>("revision")
        .expect("revision has a clap default")
}
