mod interchange;
mod io;
mod project;

use crate::arguments::CaptionCommand;
use crate::error::CliResult;

pub(crate) fn run(command: CaptionCommand) -> CliResult {
    match command {
        CaptionCommand::Import {
            input,
            format,
            timescale,
            overlap,
            namespace,
            output,
            loss_report,
            allow_lossy,
        } => interchange::import(interchange::ImportRequest {
            input: &input,
            format: format.into(),
            timescale,
            overlap_policy: overlap.into(),
            namespace: &namespace,
            output: output.as_deref(),
            loss_path: loss_report.as_deref(),
            allow_lossy,
        }),
        CaptionCommand::Export {
            document,
            format,
            output,
            loss_report,
            allow_lossy,
        } => interchange::export(
            &document,
            format.into(),
            output.as_deref(),
            loss_report.as_deref(),
            allow_lossy,
        ),
        CaptionCommand::Propose {
            project,
            document,
            bindings,
            operation_id,
            output,
        } => project::propose(
            &project,
            &document,
            &bindings,
            &operation_id,
            output.as_deref(),
        ),
        CaptionCommand::Extract {
            project,
            track,
            bindings,
            output,
        } => project::extract(&project, &track, &bindings, output.as_deref()),
    }
}
