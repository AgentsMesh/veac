use super::outcome;
use veac_artifact::ArtifactEvidenceOutcome as Artifact;
use veac_evidence::EvidenceOutcome as Evidence;

#[test]
fn every_evidence_outcome_maps_without_loss() {
    assert_eq!(outcome(Evidence::Pass), Artifact::Pass);
    assert_eq!(outcome(Evidence::Fail), Artifact::Fail);
    assert_eq!(outcome(Evidence::Error), Artifact::Error);
}
