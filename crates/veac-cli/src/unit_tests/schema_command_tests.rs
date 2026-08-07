use crate::{SchemaContract, SchemaFormat};

#[test]
fn schema_command_exposes_every_public_contract() {
    let contracts = [
        SchemaContract::Project,
        SchemaContract::Caption,
        SchemaContract::CaptionTrackInsertion,
        SchemaContract::CaptionDocumentBindings,
        SchemaContract::EditBatch,
        SchemaContract::SourceEditBatch,
        SchemaContract::SourceIndex,
        SchemaContract::BuildInputs,
        SchemaContract::EditOutcome,
        SchemaContract::RenderPlan,
        SchemaContract::Artifact,
        SchemaContract::MediaArtifactRequest,
        SchemaContract::AnalysisIngestionRequest,
        SchemaContract::AnalysisResult,
        SchemaContract::BuildManifest,
        SchemaContract::PackageManifest,
        SchemaContract::ExecutionBindings,
        SchemaContract::ProviderRequest,
        SchemaContract::ProviderResponse,
        SchemaContract::ProviderManifest,
        SchemaContract::ProviderEditProposal,
        SchemaContract::OtioLoss,
        SchemaContract::OtioImportBindings,
        SchemaContract::OtioEditProposal,
        SchemaContract::TemplateFillRequest,
        SchemaContract::LanguageSpec,
    ];
    for contract in contracts {
        let json = crate::commands::schema_json(contract, SchemaFormat::JsonSchema).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json)
            .unwrap()
            .is_object());
    }
}
