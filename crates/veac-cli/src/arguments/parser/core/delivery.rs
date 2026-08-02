use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgMatches, Command as ClapCommand};

use super::super::shared::{
    path, path_option, path_value, required_path_value, string_value, value,
};
use crate::arguments::{Command, PlanFormat, SubstitutionPolicy};

pub(super) fn commands() -> [ClapCommand; 5] {
    [plan(), manifest(), package(), render(), probe()]
}

fn config() -> Arg {
    value("config").long("config")
}

fn plan() -> ClapCommand {
    ClapCommand::new("plan")
        .about("Hydrate media facts and print one backend-neutral render plan")
        .arg(path("project"))
        .arg(config())
        .arg(path_option("bindings").long("bindings"))
        .arg(
            value("format")
                .long("format")
                .value_parser(PossibleValuesParser::new(["json"]))
                .default_value("json"),
        )
}

fn manifest() -> ClapCommand {
    ClapCommand::new("manifest")
        .about("Emit a deterministic execution manifest for one resolved output")
        .arg(path("project"))
        .arg(config())
        .arg(path_option("bindings").long("bindings"))
        .arg(path_option("output").short('o').long("output"))
}

fn package() -> ClapCommand {
    ClapCommand::new("package")
        .about("Package reachable, identity-verified inputs for one resolved output")
        .arg(path("project"))
        .arg(config())
        .arg(path_option("bindings").long("bindings"))
        .arg(
            path_option("destination")
                .short('d')
                .long("destination")
                .required(true),
        )
}

fn render() -> ClapCommand {
    ClapCommand::new("render")
        .about("Render canonical project JSON through a resolved plan and FFmpeg")
        .arg(path("project"))
        .arg(config())
        .arg(path_option("bindings").long("bindings"))
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
        .arg(path("media"))
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "plan" => Command::Plan {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            format: PlanFormat::Json,
        },
        "manifest" => Command::Manifest {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            output: path_value(matches, "output"),
        },
        "package" => Command::Package {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            destination: required_path_value(matches, "destination"),
        },
        "render" => Command::Render {
            project: required_path_value(matches, "project"),
            config: string_value(matches, "config"),
            bindings: path_value(matches, "bindings"),
            destination: path_value(matches, "destination"),
            proxy_policy: policy_value(matches, "proxy-policy"),
            render_segment_policy: policy_value(matches, "render-segment-policy"),
        },
        "probe" => Command::Probe {
            media: required_path_value(matches, "media"),
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
