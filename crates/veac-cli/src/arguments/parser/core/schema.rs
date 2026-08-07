use clap::builder::PossibleValuesParser;
use clap::{ArgMatches, Command as ClapCommand};

use super::super::shared::{required_string, value};
use crate::arguments::{Command, SchemaContract, SchemaFormat};

const CONTRACTS: [&str; 26] = [
    "project",
    "caption",
    "caption-track-insertion",
    "caption-document-bindings",
    "edit-batch",
    "source-edit-batch",
    "source-index",
    "build-inputs",
    "edit-outcome",
    "render-plan",
    "artifact",
    "media-artifact-request",
    "analysis-ingestion-request",
    "analysis-result",
    "build-manifest",
    "package-manifest",
    "execution-bindings",
    "provider-request",
    "provider-response",
    "provider-manifest",
    "provider-edit-proposal",
    "otio-loss",
    "otio-import-bindings",
    "otio-edit-proposal",
    "template-fill-request",
    "language-spec",
];

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("schema")
        .about("Print one public JSON Schema contract")
        .arg(
            value("contract")
                .long("contract")
                .value_parser(PossibleValuesParser::new(CONTRACTS))
                .default_value("project"),
        )
        .arg(
            value("format")
                .long("format")
                .value_parser(PossibleValuesParser::new(["json-schema"]))
                .default_value("json-schema"),
        )
}

pub(super) fn from_matches(matches: &ArgMatches) -> Command {
    Command::Schema {
        contract: contract(&required_string(matches, "contract")),
        format: SchemaFormat::JsonSchema,
    }
}

fn contract(value: &str) -> SchemaContract {
    match value {
        "project" => SchemaContract::Project,
        "caption" => SchemaContract::Caption,
        "caption-track-insertion" => SchemaContract::CaptionTrackInsertion,
        "caption-document-bindings" => SchemaContract::CaptionDocumentBindings,
        "edit-batch" => SchemaContract::EditBatch,
        "source-edit-batch" => SchemaContract::SourceEditBatch,
        "source-index" => SchemaContract::SourceIndex,
        "build-inputs" => SchemaContract::BuildInputs,
        "edit-outcome" => SchemaContract::EditOutcome,
        "render-plan" => SchemaContract::RenderPlan,
        "artifact" => SchemaContract::Artifact,
        "media-artifact-request" => SchemaContract::MediaArtifactRequest,
        "analysis-ingestion-request" => SchemaContract::AnalysisIngestionRequest,
        "analysis-result" => SchemaContract::AnalysisResult,
        "build-manifest" => SchemaContract::BuildManifest,
        "package-manifest" => SchemaContract::PackageManifest,
        "execution-bindings" => SchemaContract::ExecutionBindings,
        "provider-request" => SchemaContract::ProviderRequest,
        "provider-response" => SchemaContract::ProviderResponse,
        "provider-manifest" => SchemaContract::ProviderManifest,
        "provider-edit-proposal" => SchemaContract::ProviderEditProposal,
        "otio-loss" => SchemaContract::OtioLoss,
        "otio-import-bindings" => SchemaContract::OtioImportBindings,
        "otio-edit-proposal" => SchemaContract::OtioEditProposal,
        "template-fill-request" => SchemaContract::TemplateFillRequest,
        "language-spec" => SchemaContract::LanguageSpec,
        _ => unreachable!("schema contract was validated by clap"),
    }
}
