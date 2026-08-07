#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SchemaFormat {
    JsonSchema,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SchemaContract {
    Project,
    Caption,
    CaptionTrackInsertion,
    CaptionDocumentBindings,
    EditBatch,
    SourceEditBatch,
    SourceIndex,
    BuildInputs,
    EditOutcome,
    RenderPlan,
    Artifact,
    MediaArtifactRequest,
    AnalysisIngestionRequest,
    AnalysisResult,
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
    LanguageSpec,
}
