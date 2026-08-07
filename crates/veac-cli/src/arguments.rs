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
mod schema_contract;
pub(crate) use schema_contract::*;
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
    /// Ingest one closed typed provider analysis result.
    IngestAnalysis(IngestAnalysisArgs),
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
    /// Execute a programmable source entry and publish canonical project JSON.
    Build {
        source: PathBuf,
        /// Write canonical IR to PATH; omit it (or use `-`) for stdout.
        emit_ir: Option<PathBuf>,
        inputs: Option<PathBuf>,
        inline_inputs: Vec<String>,
        material_root: Option<PathBuf>,
        revision: u64,
    },
    /// Validate executable VEAC source without media I/O.
    Check {
        source: PathBuf,
        inputs: Option<PathBuf>,
        inline_inputs: Vec<String>,
        revision: u64,
    },
    /// Canonically format syntax-aware executable VEAC source.
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
    /// Print the exact revision of a `.veac` source graph.
    SourceRevision { source: PathBuf },
    /// Print the stable, agent-readable inventory of editable `.veac` source nodes.
    SourceIndex { source: PathBuf },
    /// Apply one atomic typed edit batch to `.veac` source of truth.
    SourceEdit {
        source: PathBuf,
        source_edit_batch: PathBuf,
        inputs: Option<PathBuf>,
        inline_inputs: Vec<String>,
        output: Option<PathBuf>,
        dry_run: bool,
    },
    /// Print one public JSON Schema contract.
    Schema {
        contract: SchemaContract,
        format: SchemaFormat,
    },
    /// Print the versioned VEAC language contract or its JSON Schema.
    LanguageSpec { schema: bool },
    /// Hydrate media facts and print one backend-neutral render plan.
    Plan {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        material_root: Option<PathBuf>,
        format: PlanFormat,
    },
    /// Emit a deterministic execution manifest for one resolved output.
    Manifest {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        material_root: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    /// Package reachable, identity-verified inputs for one resolved output.
    Package {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        material_root: Option<PathBuf>,
        destination: PathBuf,
    },
    /// Render canonical project JSON through a resolved plan and FFmpeg.
    Render {
        project: PathBuf,
        config: Option<String>,
        bindings: Option<PathBuf>,
        material_root: Option<PathBuf>,
        /// Place every authored deliverable file name in this existing directory.
        destination: Option<PathBuf>,
        proxy_policy: SubstitutionPolicy,
        render_segment_policy: SubstitutionPolicy,
    },
    /// Print a normalized canonical media probe snapshot.
    Probe {
        input: PathBuf,
        material: Option<String>,
        material_root: Option<PathBuf>,
    },
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
