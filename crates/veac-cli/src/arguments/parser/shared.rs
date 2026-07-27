use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches};

pub(super) fn path(id: &'static str) -> Arg {
    value(id)
        .required(true)
        .value_parser(clap::value_parser!(PathBuf))
}

pub(super) fn path_option(id: &'static str) -> Arg {
    value(id).value_parser(clap::value_parser!(PathBuf))
}

pub(super) fn value(id: &'static str) -> Arg {
    Arg::new(id).value_name(value_name(id))
}

pub(super) fn flag(id: &'static str, long: &'static str) -> Arg {
    Arg::new(id).long(long).action(ArgAction::SetTrue)
}

pub(super) fn required_path_value(matches: &ArgMatches, id: &str) -> PathBuf {
    matches
        .get_one::<PathBuf>(id)
        .expect("required path was validated by clap")
        .clone()
}

pub(super) fn path_value(matches: &ArgMatches, id: &str) -> Option<PathBuf> {
    matches.get_one::<PathBuf>(id).cloned()
}

pub(super) fn required_string(matches: &ArgMatches, id: &str) -> String {
    matches
        .get_one::<String>(id)
        .expect("required string was validated by clap")
        .clone()
}

pub(super) fn string_value(matches: &ArgMatches, id: &str) -> Option<String> {
    matches.get_one::<String>(id).cloned()
}

pub(super) fn flag_value(matches: &ArgMatches, id: &str) -> bool {
    matches.get_flag(id)
}

fn value_name(id: &str) -> &'static str {
    match id {
        "input" => "INPUT",
        "spec" => "SPEC",
        "store" => "STORE",
        "ffmpeg" => "FFMPEG",
        "ffprobe" => "FFPROBE",
        "request" => "REQUEST",
        "program" => "PROGRAM",
        "response" => "RESPONSE",
        "project" => "PROJECT",
        "context" => "CONTEXT",
        "output" => "OUTPUT",
        "format" => "FORMAT",
        "timescale" => "TIMESCALE",
        "overlap" => "OVERLAP",
        "namespace" => "NAMESPACE",
        "loss_report" => "LOSS_REPORT",
        "document" => "DOCUMENT",
        "operation_id" => "OPERATION_ID",
        "bindings" => "BINDINGS",
        "key" => "KEY",
        "package" => "PACKAGE",
        "search" => "SEARCH",
        "track" => "TRACK",
        "sequence" => "SEQUENCE",
        "timeline" => "TIMELINE",
        "source" => "SOURCE",
        "emit_ir" => "EMIT_IR",
        "revision" => "REVISION",
        "edit_batch" => "EDIT_BATCH",
        "contract" => "CONTRACT",
        "config" => "CONFIG",
        "destination" => "DESTINATION",
        "media" => "MEDIA",
        _ => unreachable!("all parser value identifiers have display names"),
    }
}
