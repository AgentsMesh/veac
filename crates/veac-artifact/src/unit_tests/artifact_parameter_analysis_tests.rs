use crate::*;

#[test]
fn analysis_parameters_can_bind_a_verified_result_digest() {
    let result = AnalysisResultEnvelope::new(
        AnalysisDescriptor::SceneBoundaries(SceneBoundaryAnalysisDescriptor {
            sensitivity_millionths: 500_000,
        }),
        AnalysisResult::SceneBoundaries(SceneBoundaryAnalysisResult {
            boundaries: Vec::new(),
        }),
    )
    .unwrap();
    let payload = result.canonical_bytes(MAX_ANALYSIS_PAYLOAD_BYTES).unwrap();
    let parameters = ArtifactParameters::Analysis(AnalysisArtifactParameters {
        descriptor: result.descriptor,
        result_digest: ContentDigest::sha256(payload),
    });

    assert_eq!(parameters.kind(), ArtifactKind::Analysis);
    parameters.validate().unwrap();
}
