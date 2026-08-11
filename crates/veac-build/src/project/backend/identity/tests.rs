use super::*;

fn digest(value: u8) -> ContentDigest {
    ContentDigest::sha256([value])
}

fn render() -> ProjectBackendIdentity {
    ProjectBackendIdentity::VeacRender {
        project_backend: digest(1),
        build: digest(2),
        compiler: digest(3),
        codegen: digest(4),
        runtime: digest(5),
        ffmpeg: digest(6),
    }
}

#[test]
fn identity_is_closed_validated_and_action_specific() {
    let baseline = render();
    let first = baseline.digest_for(ProjectActionKind::VeacRender).unwrap();
    assert_eq!(
        first,
        baseline.digest_for(ProjectActionKind::VeacRender).unwrap()
    );
    assert!(baseline.digest_for(ProjectActionKind::Evidence).is_err());

    let mut changed = render();
    if let ProjectBackendIdentity::VeacRender { runtime, .. } = &mut changed {
        *runtime = digest(9);
    }
    assert_ne!(
        first,
        changed.digest_for(ProjectActionKind::VeacRender).unwrap()
    );
    if let ProjectBackendIdentity::VeacRender { runtime, .. } = &mut changed {
        runtime.value = "invalid".to_owned();
    }
    assert!(changed.digest_for(ProjectActionKind::VeacRender).is_err());
}

#[test]
fn every_action_variant_has_a_distinct_identity_domain() {
    let common = || digest(7);
    let derivation = ProjectBackendIdentity::MediaDerivation {
        project_backend: common(),
        build: common(),
        codegen: common(),
        runtime: common(),
        ffmpeg: common(),
        ffprobe: common(),
    };
    let evidence = ProjectBackendIdentity::Evidence {
        project_backend: common(),
        build: common(),
        compiler: common(),
        evidence: common(),
        runtime: common(),
        ffmpeg: common(),
        ffprobe: common(),
    };
    assert_ne!(
        derivation
            .digest_for(ProjectActionKind::MediaDerivation)
            .unwrap(),
        evidence.digest_for(ProjectActionKind::Evidence).unwrap()
    );
}
