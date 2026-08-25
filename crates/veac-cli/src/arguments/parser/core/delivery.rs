use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgMatches, Command as ClapCommand};

use super::super::shared::{
    material_root, path, path_option, path_value, required_path_value, string_value, value,
};
use crate::arguments::{Command, PlanFormat, SubstitutionPolicy};

pub(super) fn commands() -> [ClapCommand; 5] {
    [plan(), manifest(), bundle(), render(), probe()]
}

fn config() -> Arg {
    value("config").long("config")
}

fn input_resolution(command: ClapCommand) -> ClapCommand {
    command
        .arg(path_option("bindings").long("bindings"))
        .arg(material_root().conflicts_with("bindings"))
}

fn plan() -> ClapCommand {
    input_resolution(
        ClapCommand::new("plan")
            .about("Hydrate media facts and print one backend-neutral render plan")
            .arg(path("project"))
            .arg(config()),
    )
    .arg(
        value("format")
            .long("format")
            .value_parser(PossibleValuesParser::new(["json"]))
            .default_value("json"),
    )
}

fn manifest() -> ClapCommand {
    input_resolution(
        ClapCommand::new("manifest")
            .about("Emit a deterministic execution manifest for one resolved output")
            .arg(path("project"))
            .arg(config()),
    )
    .arg(path_option("output").short('o').long("output"))
}

fn bundle() -> ClapCommand {
    input_resolution(
        ClapCommand::new("bundle")
            .about("Bundle reachable, identity-verified inputs for one resolved output")
            .arg(path("project"))
            .arg(config()),
    )
    .arg(
        path_option("destination")
            .short('d')
            .long("destination")
            .required(true),
    )
}

fn render() -> ClapCommand {
    input_resolution(
        ClapCommand::new("render")
            .about("Render canonical project JSON through a resolved plan and FFmpeg")
            .arg(path("project"))
            .arg(config()),
    )
    .arg(policy("proxy-policy"))
    .arg(policy("render-segment-policy"))
    .arg(
        path_option("destination")
            .short('d')
            .long("destination")
            .help("Place every authored deliverable file name in this existing directory"),
    )
}

fn policy(name: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .value_name("POLICY")
        .value_parser(PossibleValuesParser::new(["original", "prefer", "require"]))
        .default_value("prefer")
}

fn probe() -> ClapCommand {
    ClapCommand::new("probe")
        .about("Print a normalized canonical media probe snapshot")
        .arg(path("input").help("Media file, or canonical project when --material is set"))
        .arg(
            value("material")
                .long("material")
                .help("Probe one material using its canonical URI, identity, and stream intent"),
        )
        .arg(material_root().requires("material"))
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "plan" => Command::Plan {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            material_root: path_value(matches, "material_root"),
            format: PlanFormat::Json,
        },
        "manifest" => Command::Manifest {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            material_root: path_value(matches, "material_root"),
            output: path_value(matches, "output"),
        },
        "bundle" => Command::Bundle {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            material_root: path_value(matches, "material_root"),
            destination: required_path_value(matches, "destination"),
        },
        "render" => Command::Render {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            material_root: path_value(matches, "material_root"),
            destination: path_value(matches, "destination"),
            proxy_policy: policy_value(matches, "proxy-policy"),
            render_segment_policy: policy_value(matches, "render-segment-policy"),
        },
        "probe" => Command::Probe {
            input: required_path_value(matches, "input"),
            material: string_value(matches, "material"),
            material_root: path_value(matches, "material_root"),
        },
        _ => unreachable!("clap only accepts registered delivery commands"),
    }
}

fn policy_value(matches: &ArgMatches, name: &str) -> SubstitutionPolicy {
    match matches.get_one::<String>(name).map(String::as_str) {
        Some("original") => SubstitutionPolicy::Original,
        Some("prefer") => SubstitutionPolicy::Prefer,
        Some("require") => SubstitutionPolicy::Require,
        _ => unreachable!("clap validates substitution policies"),
    }
}
