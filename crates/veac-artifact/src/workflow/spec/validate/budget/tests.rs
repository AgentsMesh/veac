use super::*;

#[test]
fn artifact_range_addition_overflow_is_a_resource_limit() {
    let maximum = RationalTime::new(veac_ir::MAX_SAFE_INTEGER as i64, 1).unwrap();
    let limits = MediaArtifactLimits {
        max_duration_seconds: u64::MAX,
        ..MediaArtifactLimits::default()
    };
    assert_eq!(
        range(maximum, maximum, limits).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}
