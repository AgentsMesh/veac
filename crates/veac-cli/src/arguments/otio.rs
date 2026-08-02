use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum OtioCommand {
    /// Export one sequence as standard OTIO plus the lossless VEAC extension.
    Export {
        project: PathBuf,
        sequence: Option<String>,
        output: Option<PathBuf>,
        loss_report: Option<PathBuf>,
        allow_lossy: bool,
    },
    /// Convert OTIO into a revision-checked, atomic canonical edit proposal.
    Propose {
        project: PathBuf,
        timeline: PathBuf,
        /// Required for third-party OTIO without a VEAC metadata extension.
        bindings: Option<PathBuf>,
        operation_id: String,
        output: Option<PathBuf>,
        loss_report: Option<PathBuf>,
        allow_lossy: bool,
    },
}
