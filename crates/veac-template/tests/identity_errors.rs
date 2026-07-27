#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::{FillMode, HashAlgorithm};
use veac_template::TemplateErrorKind;

#[test]
fn replacement_requires_a_content_identity() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.identity = None;
    assert_eq!(
        error_kind(&project, &request(&project, replacement)),
        TemplateErrorKind::MissingIdentity
    );
}

#[test]
fn probe_facts_must_match_a_sha256_identity() {
    let project = project(FillMode::FitDuration, false);
    let mut mismatch = video(six_seconds(), 1920, 1080);
    mismatch.probe.as_mut().unwrap().observed_identity.digest = "c".repeat(64);
    assert_eq!(
        error_kind(&project, &request(&project, mismatch)),
        TemplateErrorKind::ProbeIdentityMismatch
    );

    let mut unsupported = video(six_seconds(), 1920, 1080);
    unsupported.identity.as_mut().unwrap().algorithm = HashAlgorithm::Blake3;
    unsupported
        .probe
        .as_mut()
        .unwrap()
        .observed_identity
        .algorithm = HashAlgorithm::Blake3;
    assert_eq!(
        error_kind(&project, &request(&project, unsupported)),
        TemplateErrorKind::ProbeIdentityMismatch
    );
}
