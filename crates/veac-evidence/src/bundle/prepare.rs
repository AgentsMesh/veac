use serde::Serialize;

use super::derivative;
use crate::{
    build_bundle_manifest, canonical_json, evaluate, sha256_hex, BundleArtifactContent,
    BundleError, DecodeObservation, EvidenceProvenanceV1, ObservationPlanV1, ObservationRun,
    PreparedEvidenceBundle, RationalTime, ValidatedSuite,
};

#[derive(Serialize)]
struct ObservationSummary<'a> {
    schema_version: u32,
    frames: Vec<FrameSummary<'a>>,
    decodes: &'a std::collections::BTreeMap<String, DecodeObservation>,
    layer_orders: &'a std::collections::BTreeMap<String, crate::LayerOrderObservation>,
    failures: &'a [crate::ObservationFailure],
}

#[derive(Serialize)]
struct FrameSummary<'a> {
    sample_id: &'a str,
    width: u32,
    height: u32,
    actual_pts: RationalTime,
    png: String,
}

pub fn prepare_evidence_bundle(
    suite: &ValidatedSuite,
    plan: &ObservationPlanV1,
    run: &ObservationRun,
    provenance: &EvidenceProvenanceV1,
) -> Result<PreparedEvidenceBundle, BundleError> {
    validate_contract(suite, plan, provenance)?;
    let report = evaluate(suite, &run.observations);
    let mut artifacts = vec![
        content("suite.json", canonical_json(suite.as_suite())?),
        content("observation-plan.json", canonical_json(plan)?),
        content("provenance.json", canonical_json(provenance)?),
        content("results.json", canonical_json(&report)?),
        content("results.csv", derivative::csv(&report)),
        content("junit.xml", derivative::junit(&report)),
        content("index.html", derivative::html(&report)),
    ];
    let mut frames = Vec::with_capacity(run.observations.frames.len());
    for (id, frame) in &run.observations.frames {
        let path = format!("frames/{id}.png");
        artifacts.push(content(&path, derivative::frame(frame)?));
        frames.push(FrameSummary {
            sample_id: id,
            width: frame.width,
            height: frame.height,
            actual_pts: frame.actual_pts,
            png: path,
        });
    }
    if let Some(sheet) = derivative::contact_sheet(&run.observations.frames)? {
        artifacts.push(content("contact-sheet.png", sheet));
    }
    let summary = ObservationSummary {
        schema_version: 1,
        frames,
        decodes: &run.observations.decodes,
        layer_orders: &run.observations.layer_orders,
        failures: &run.failures,
    };
    artifacts.push(content("observations.json", canonical_json(&summary)?));
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = build_bundle_manifest(suite.as_suite(), &report, provenance, artifacts.clone())?;
    Ok(PreparedEvidenceBundle {
        manifest,
        artifacts,
    })
}

fn validate_contract(
    suite: &ValidatedSuite,
    plan: &ObservationPlanV1,
    provenance: &EvidenceProvenanceV1,
) -> Result<(), BundleError> {
    let suite_digest = sha256_hex(&canonical_json(suite.as_suite())?);
    let plan_digest = sha256_hex(&canonical_json(plan)?);
    if plan.suite_sha256 != suite_digest
        || provenance.suite_sha256 != suite_digest
        || provenance.observation_plan_sha256 != plan_digest
    {
        return Err(BundleError::InvalidContract(
            "suite, observation plan, and provenance identities disagree".into(),
        ));
    }
    Ok(())
}

fn content(path: &str, bytes: Vec<u8>) -> BundleArtifactContent {
    BundleArtifactContent {
        path: path.to_owned(),
        bytes,
    }
}
