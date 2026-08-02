use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum CaptionCommand {
    /// Convert an SRT, WebVTT, or ASS sidecar to a canonical caption document.
    Import {
        input: PathBuf,
        format: CaptionFormatArg,
        timescale: u32,
        overlap: CaptionOverlapArg,
        namespace: String,
        output: Option<PathBuf>,
        loss_report: Option<PathBuf>,
        allow_lossy: bool,
    },
    /// Convert a canonical caption document to an SRT, WebVTT, or ASS sidecar.
    Export {
        document: PathBuf,
        format: CaptionFormatArg,
        output: Option<PathBuf>,
        loss_report: Option<PathBuf>,
        allow_lossy: bool,
    },
    /// Produce a checked canonical EditBatch that inserts one caption track.
    Propose {
        project: PathBuf,
        document: PathBuf,
        bindings: PathBuf,
        operation_id: String,
        output: Option<PathBuf>,
    },
    /// Extract one caption track into a canonical caption document.
    Extract {
        project: PathBuf,
        track: String,
        bindings: PathBuf,
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptionFormatArg {
    Srt,
    WebVtt,
    Ass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptionOverlapArg {
    Reject,
    Allow,
}

impl From<CaptionFormatArg> for veac_caption::CaptionFormat {
    fn from(value: CaptionFormatArg) -> Self {
        match value {
            CaptionFormatArg::Srt => Self::Srt,
            CaptionFormatArg::WebVtt => Self::WebVtt,
            CaptionFormatArg::Ass => Self::Ass,
        }
    }
}

impl From<CaptionOverlapArg> for veac_caption::OverlapPolicy {
    fn from(value: CaptionOverlapArg) -> Self {
        match value {
            CaptionOverlapArg::Reject => Self::Reject,
            CaptionOverlapArg::Allow => Self::Allow,
        }
    }
}
