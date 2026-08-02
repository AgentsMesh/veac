use super::*;
use crate::{test_support, VerifiedSourceCopy};

#[test]
fn every_discovery_budget_arithmetic_overflow_fails_closed() {
    let overflowed_deadline = RelinkDiscoveryLimits {
        max_wall_time: std::time::Duration::MAX,
        ..RelinkDiscoveryLimits::default()
    };
    assert_eq!(
        DiscoveryBudget::new(overflowed_deadline, Instant::now())
            .err()
            .unwrap()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );

    let mut entries = budget();
    entries.entries = usize::MAX;
    assert_limit(entries.observe_entry());

    let mut bytes = budget();
    bytes.hashed_bytes = bytes.limits.max_total_bytes + 1;
    assert_limit(bytes.file_limit());

    let mut hashed = budget();
    hashed.hashed_bytes = u64::MAX;
    assert_limit(hashed.candidate(
        "candidate".into(),
        VerifiedSourceCopy {
            identity: test_support::media_identity(b"x"),
            size_bytes: 1,
        },
    ));
}

fn budget() -> DiscoveryBudget {
    DiscoveryBudget::new(RelinkDiscoveryLimits::default(), Instant::now()).unwrap()
}

fn assert_limit<T>(result: ArtifactResult<T>) {
    assert_eq!(result.err().unwrap().kind, ArtifactErrorKind::ResourceLimit);
}
