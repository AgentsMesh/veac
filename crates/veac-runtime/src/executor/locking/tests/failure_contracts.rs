use rustix::io::Errno;

use super::super::{lock, FailurePoint, Operations};
use super::acquire;

struct FailAt(FailurePoint);

impl Operations for FailAt {
    fn before(&self, point: FailurePoint) -> Result<(), Errno> {
        if self.0 == point {
            Err(Errno::IO)
        } else {
            Ok(())
        }
    }
}

#[test]
fn descriptor_inspection_failures_keep_the_directory_locked() {
    for (point, action, has_path_context) in [
        (
            FailurePoint::InspectLockedDirectory,
            "inspect locked output directory",
            true,
        ),
        (
            FailurePoint::InspectCurrentDirectory,
            "inspect current output directory",
            true,
        ),
        (
            FailurePoint::DuplicateDirectory,
            "duplicate locked output directory",
            false,
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let parent = std::fs::canonicalize(temp.path()).unwrap();
        let locks = acquire(std::slice::from_ref(&parent)).unwrap();

        let error = locks
            .directory_at_canonical_path(&parent, &FailAt(point))
            .unwrap_err();
        assert!(error.message.contains(action), "{}", error.message);
        if has_path_context {
            assert!(error.message.contains(&parent.display().to_string()));
        }
        assert!(acquire(std::slice::from_ref(&parent))
            .unwrap_err()
            .message
            .contains("locked by another VEAC render"));

        drop(locks);
        acquire(&[parent]).unwrap();
    }
}

#[test]
fn lock_descriptor_failures_release_resources_for_retry() {
    for (point, action) in [
        (FailurePoint::InspectLock, "inspect render lock"),
        (FailurePoint::AcquireLock, "acquire render lock"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let parent = std::fs::canonicalize(temp.path()).unwrap();

        let error = lock(&parent, &FailAt(point)).unwrap_err();
        assert!(error.message.contains(action), "{}", error.message);
        assert!(error.message.contains(&parent.display().to_string()));
        acquire(&[parent]).unwrap();
    }
}
