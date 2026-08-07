use crate::error::{CliError, CliResult};
use crate::{SchemaContract, SchemaFormat};

pub(crate) fn run(contract: SchemaContract, format: SchemaFormat) -> CliResult {
    crate::fs::write_stdout(&encode(contract, format)?)
}

pub(crate) fn encode(contract: SchemaContract, format: SchemaFormat) -> CliResult<String> {
    let schema = match (contract, format) {
        (SchemaContract::Project, SchemaFormat::JsonSchema) => {
            convert(veac_ir::project_json_schema())
        }
        (SchemaContract::Caption, SchemaFormat::JsonSchema) => {
            convert(veac_caption::caption_json_schema())
        }
        (SchemaContract::CaptionTrackInsertion, SchemaFormat::JsonSchema) => {
            convert(veac_caption::caption_track_insertion_json_schema())
        }
        (SchemaContract::CaptionDocumentBindings, SchemaFormat::JsonSchema) => {
            convert(veac_caption::caption_document_bindings_json_schema())
        }
        (SchemaContract::EditBatch, SchemaFormat::JsonSchema) => {
            convert(veac_ir::edit_batch_json_schema())
        }
        (SchemaContract::SourceEditBatch, SchemaFormat::JsonSchema) => {
            convert(veac_lang::source_edit::source_edit_batch_json_schema())
        }
        (SchemaContract::SourceIndex, SchemaFormat::JsonSchema) => {
            convert(veac_lang::program::source_index_json_schema())
        }
        (SchemaContract::BuildInputs, SchemaFormat::JsonSchema) => {
            convert(veac_lang::program::build_input_manifest_json_schema())
        }
        (SchemaContract::EditOutcome, SchemaFormat::JsonSchema) => {
            convert(veac_ir::edit_outcome_json_schema())
        }
        (SchemaContract::RenderPlan, SchemaFormat::JsonSchema) => {
            convert(veac_plan::render_plan_json_schema())
        }
        (SchemaContract::Artifact, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::artifact_descriptor_json_schema())
        }
        (SchemaContract::MediaArtifactRequest, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::media_artifact_request_json_schema())
        }
        (SchemaContract::AnalysisIngestionRequest, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::analysis_ingestion_request_json_schema())
        }
        (SchemaContract::AnalysisResult, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::analysis_result_json_schema())
        }
        (SchemaContract::BuildManifest, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::build_manifest_json_schema())
        }
        (SchemaContract::PackageManifest, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::package_manifest_json_schema())
        }
        (SchemaContract::ExecutionBindings, SchemaFormat::JsonSchema) => {
            convert(veac_artifact::execution_binding_manifest_json_schema())
        }
        (SchemaContract::ProviderRequest, SchemaFormat::JsonSchema) => {
            convert(veac_provider::provider_request_json_schema())
        }
        (SchemaContract::ProviderResponse, SchemaFormat::JsonSchema) => {
            convert(veac_provider::provider_response_json_schema())
        }
        (SchemaContract::ProviderManifest, SchemaFormat::JsonSchema) => {
            convert(veac_provider::provider_manifest_json_schema())
        }
        (SchemaContract::ProviderEditProposal, SchemaFormat::JsonSchema) => {
            convert(veac_provider::provider_edit_proposal_json_schema())
        }
        (SchemaContract::OtioLoss, SchemaFormat::JsonSchema) => {
            convert(veac_otio::otio_loss_json_schema())
        }
        (SchemaContract::OtioImportBindings, SchemaFormat::JsonSchema) => {
            convert(veac_otio::otio_import_bindings_json_schema())
        }
        (SchemaContract::OtioEditProposal, SchemaFormat::JsonSchema) => {
            convert(veac_otio::otio_edit_proposal_json_schema())
        }
        (SchemaContract::TemplateFillRequest, SchemaFormat::JsonSchema) => {
            convert(veac_template::template_fill_request_json_schema())
        }
        (SchemaContract::LanguageSpec, SchemaFormat::JsonSchema) => {
            convert(veac_lang::vocabulary::language_spec_json_schema())
        }
    };
    let schema = schema?;
    let encoded = if contract == SchemaContract::LanguageSpec {
        serde_json_canonicalizer::to_string(&schema)
    } else {
        serde_json::to_string_pretty(&schema)
    };
    let mut json = encoded.map_err(|error| CliError::new("SCHEMA_ENCODE", error.to_string()))?;
    json.push('\n');
    Ok(json)
}

fn convert<T, E: std::fmt::Display>(value: Result<T, E>) -> CliResult<T> {
    match value {
        Ok(value) => Ok(value),
        Err(error) => Err(CliError::new("SCHEMA_ENCODE", error.to_string())),
    }
}
