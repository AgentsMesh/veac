mod export;
mod io;
mod propose;

use crate::arguments::OtioCommand;
use crate::error::CliResult;

pub(crate) fn run(command: OtioCommand) -> CliResult {
    match command {
        OtioCommand::Export {
            project,
            sequence,
            output,
            loss_report,
            allow_lossy,
        } => export::run(
            &project,
            sequence.as_deref(),
            output.as_deref(),
            loss_report.as_deref(),
            allow_lossy,
        ),
        OtioCommand::Propose {
            project,
            timeline,
            bindings,
            operation_id,
            output,
            loss_report,
            allow_lossy,
        } => propose::run(propose::Request {
            project: &project,
            timeline: &timeline,
            bindings: bindings.as_deref(),
            operation_id: &operation_id,
            output: output.as_deref(),
            loss_report: loss_report.as_deref(),
            allow_lossy,
        }),
    }
}
