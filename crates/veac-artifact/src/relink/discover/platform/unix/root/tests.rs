use std::time::Instant;

use super::*;
use crate::RelinkDiscoveryLimits;

#[test]
fn absolute_binding_rejects_relative_and_parent_components() {
    let budget = DiscoveryBudget::new(RelinkDiscoveryLimits::default(), Instant::now()).unwrap();
    for path in [Path::new("relative"), Path::new("/..")] {
        assert_eq!(
            bind_absolute(path, &budget).unwrap_err().kind,
            ArtifactErrorKind::UnsafePath
        );
    }
}
