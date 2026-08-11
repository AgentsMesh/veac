use schemars::schema_for;

pub fn evidence_suite_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(crate::EvidenceSuiteV1))
}

pub fn observation_plan_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(crate::ObservationPlanV1))
}

pub fn evidence_report_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(crate::EvidenceReportV1))
}

pub fn evidence_bundle_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(crate::EvidenceBundleManifestV1))
}

pub fn evidence_provenance_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(crate::EvidenceProvenanceV1))
}
