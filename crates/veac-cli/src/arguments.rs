use std::ffi::OsString;
use std::path::PathBuf;

use crate::diagnostic::DiagnosticFormat;

mod caption;
pub(crate) use caption::*;
mod artifact;
pub(crate) use artifact::*;
mod otio;
pub(crate) use otio::*;
mod parser;
mod template;
pub(crate) use template::*;
mod workflow;
pub(crate) use workflow::*;

#[derive(Debug)]
pub(super) struct Cli {
    pub(super) diagnostic_format: DiagnosticFormat,
    pub(super) command: Command,
}

impl Cli {
    pub(super) fn parse() -> Self {
        let matches = parser::command().get_matches();
        Self {
            diagnostic_format: diagnostic_format(&matches),
            command: parser::from_matches(&matches),
        }
    }

    pub(super) fn try_parse_from<I, T>(arguments: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = parser::command().try_get_matches_from(arguments)?;
        Ok(Self {
            diagnostic_format: diagnostic_format(&matches),
            command: parser::from_matches(&matches),
        })
    }
}

fn diagnostic_format(matches: &clap::ArgMatches) -> DiagnosticFormat {
    match matches
        .get_one::<String>("diagnostic_format")
        .map(String::as_str)
    {
        Some("json") => DiagnosticFormat::Json,
        _ => DiagnosticFormat::Human,
    }
}

#[derive(Debug)]
pub(super) enum Command {
    /// Loss-aware caption sidecar interchange and canonical edit proposals.
    Caption { command: CaptionCommand },
    /// Loss-aware OpenTimelineIO interchange and canonical edit proposals.
    Otio { command: OtioCommand },
    /// Fill typed media and text template slots through a canonical edit proposal.
    Template { command: TemplateCommand },
    /// Derive one content-addressed media artifact with FFmpeg.
    Derive(DeriveArgs),
    /// Execute one deterministic external provider request.
    ProviderRun(ProviderRunArgs),
    /// Convert a provider response into a reviewable canonical edit proposal.
    ProviderPropose(ProviderProposeArgs),
    /// Inspect or remove one verified content-addressed artifact.
    Artifact { command: ArtifactCommand },
    /// Produce verified machine-local bindings from a portable package.
    PackageBindings(PackageBindingsArgs),
    /// Discover identity-matched replacement files for one canonical plan.
    Relink(RelinkArgs),
    /// Lower agent-oriented source into canonical project JSON.
    Compile {
        source: PathBuf,
        /// Write canonical IR to PATH; omit it (or use `-`) for stdout.
        emit_ir: Option<PathBuf>,
        revision: u64,
    },
    /// Validate agent-oriented source without media I/O.
    Check { source: PathBuf, revision: u64 },
    /// Canonically format agent-oriented source.
    Fmt {
        source: PathBuf,
        check: bool,
        stdout: bool,
    },
    /// Validate strict canonical project JSON.
    CheckIr { project: PathBuf },
    /// Apply one atomic typed edit batch to canonical project JSON.
    Edit {
        project: PathBuf,
        edit_batch: PathBuf,
        /// Write the accepted project to PATH; defaults to updating PROJECT in place.
        output: Option<PathBuf>,
        /// Compute and emit the outcome without writing a project.
        dry_run: bool,
    },
    /// Print one public JSON Schema contract.
    Schema {
        contract: SchemaContract,
        format: SchemaFormat,
    },
    /// Hydrate media facts and print one backend-neutral render plan.
    Plan {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        format: PlanFormat,
    },
    /// Emit a deterministic execution manifest for one resolved output.
    Manifest {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    /// Package reachable, identity-verified inputs for one resolved output.
    Package {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        destination: PathBuf,
    },
    /// Render canonical project JSON through a resolved plan and FFmpeg.
    Render {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        /// Place every authored deliverable file name in this existing directory.
        destination: Option<PathBuf>,
        proxy_policy: SubstitutionPolicy,
        render_segment_policy: SubstitutionPolicy,
    },
    /// Print a normalized canonical media probe snapshot.
    Probe { media: PathBuf },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SubstitutionPolicy {
    Original,
    Prefer,
    Require,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlanFormat {
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SchemaFormat {
    JsonSchema,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SchemaContract {
    Project,
    Caption,
    CaptionTrackInsertion,
    CaptionDocumentBindings,
    EditBatch,
    EditOutcome,
    RenderPlan,
    Artifact,
    MediaArtifactRequest,
    BuildManifest,
    PackageManifest,
    ExecutionBindings,
    ProviderRequest,
    ProviderResponse,
    ProviderManifest,
    ProviderEditProposal,
    OtioLoss,
    OtioImportBindings,
    OtioEditProposal,
    TemplateFillRequest,
}
