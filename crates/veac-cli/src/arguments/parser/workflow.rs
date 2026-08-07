use clap::{Arg, ArgAction, ArgMatches, Command as ClapCommand};

use super::shared::{path, path_option, path_value, required_path_value};
use crate::arguments::{
    Command, DeriveArgs, IngestAnalysisArgs, ProviderProposeArgs, ProviderRunArgs,
};

pub(super) fn commands() -> [ClapCommand; 4] {
    [
        derive(),
        ingest_analysis(),
        provider_run(),
        provider_propose(),
    ]
}

fn ingest_analysis() -> ClapCommand {
    ClapCommand::new("ingest-analysis")
        .about("Ingest one closed typed provider analysis result")
        .arg(path("input"))
        .arg(path("request"))
        .arg(path_option("store").long("store").required(true))
}

fn derive() -> ClapCommand {
    ClapCommand::new("derive")
        .about("Derive one content-addressed media artifact with FFmpeg")
        .arg(path("input"))
        .arg(path("spec"))
        .arg(path_option("store").long("store").required(true))
        .arg(path_option("ffmpeg").long("ffmpeg").default_value("ffmpeg"))
        .arg(
            path_option("ffprobe")
                .long("ffprobe")
                .default_value("ffprobe"),
        )
}

fn provider_run() -> ClapCommand {
    ClapCommand::new("provider-run")
        .about("Execute one deterministic external provider request")
        .arg(path("request"))
        .arg(path_option("program").long("program").required(true))
        .arg(path_option("store").long("store").required(true))
        .arg(
            Arg::new("arguments")
                .long("arg")
                .value_name("ARGUMENTS")
                .allow_hyphen_values(true)
                .action(ArgAction::Append),
        )
        .arg(path_option("response").long("response"))
}

fn provider_propose() -> ClapCommand {
    ClapCommand::new("provider-propose")
        .about("Convert a provider response into a reviewable canonical edit proposal")
        .arg(path("project"))
        .arg(path("request"))
        .arg(path("response"))
        .arg(path("context"))
        .arg(path_option("output").short('o').long("output"))
}

pub(super) fn from_matches(name: &str, matches: &ArgMatches) -> Command {
    match name {
        "derive" => Command::Derive(DeriveArgs {
            input: required_path_value(matches, "input"),
            spec: required_path_value(matches, "spec"),
            store: required_path_value(matches, "store"),
            ffmpeg: required_path_value(matches, "ffmpeg"),
            ffprobe: required_path_value(matches, "ffprobe"),
        }),
        "ingest-analysis" => Command::IngestAnalysis(IngestAnalysisArgs {
            input: required_path_value(matches, "input"),
            request: required_path_value(matches, "request"),
            store: required_path_value(matches, "store"),
        }),
        "provider-run" => Command::ProviderRun(ProviderRunArgs {
            request: required_path_value(matches, "request"),
            program: required_path_value(matches, "program"),
            store: required_path_value(matches, "store"),
            arguments: matches
                .get_many::<String>("arguments")
                .map(|values| values.cloned().collect())
                .unwrap_or_default(),
            response: path_value(matches, "response"),
        }),
        "provider-propose" => Command::ProviderPropose(ProviderProposeArgs {
            project: required_path_value(matches, "project"),
            request: required_path_value(matches, "request"),
            response: required_path_value(matches, "response"),
            context: required_path_value(matches, "context"),
            output: path_value(matches, "output"),
        }),
        _ => unreachable!("clap only accepts registered workflow commands"),
    }
}
