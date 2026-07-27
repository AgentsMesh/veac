use veac_codegen::emitter::BackendResource;

use super::super::resource;
use super::support::{bundle, fake_identity, identity};

#[test]
fn resources_validate_hash_and_reject_identity_changes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("input.bin");
    std::fs::write(&path, b"version-one").unwrap();
    let mut bundle = bundle(&temp.path().join("output.srt"));
    bundle.protected_resources.push(BackendResource {
        path: path.clone(),
        expected_identity: identity(&path),
    });
    let validated = validate(&bundle).unwrap();
    assert_eq!(validated.paths.len(), 1);
    resource::verify_until(&bundle, &validated.fingerprint, deadline()).unwrap();

    std::fs::write(&path, b"version-two").unwrap();
    let error = resource::verify_until(&bundle, &validated.fingerprint, deadline()).unwrap_err();
    assert!(error.message.contains("identity changed"));
}

#[test]
fn resource_fingerprint_detects_a_different_valid_set() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first.bin");
    let second = temp.path().join("second.bin");
    std::fs::write(&first, b"first").unwrap();
    std::fs::write(&second, b"second").unwrap();
    let mut original = bundle(&temp.path().join("output.srt"));
    original.protected_resources.push(BackendResource {
        path: first.clone(),
        expected_identity: identity(&first),
    });
    let fingerprint = validate(&original).unwrap().fingerprint;
    original.protected_resources[0] = BackendResource {
        path: second.clone(),
        expected_identity: identity(&second),
    };
    assert!(resource::verify_until(&original, &fingerprint, deadline())
        .unwrap_err()
        .message
        .contains("paths changed"));
}

fn deadline() -> std::time::Instant {
    std::time::Instant::now() + std::time::Duration::from_secs(10)
}

#[test]
fn resources_reject_missing_unsafe_and_invalid_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.srt");
    let cases = [
        (
            temp.path().join("missing"),
            fake_identity('a'),
            "cannot validate",
        ),
        (
            temp.path().to_path_buf(),
            fake_identity('a'),
            "regular non-symlink",
        ),
        (
            temp.path().join("bad"),
            fake_identity('A'),
            "lowercase SHA-256",
        ),
        (
            temp.path().join("short"),
            fake_identity('a'),
            "lowercase SHA-256",
        ),
    ];
    for (index, (path, mut expected_identity, message)) in cases.into_iter().enumerate() {
        if index == 2 {
            std::fs::write(&path, b"value").unwrap();
        }
        if index == 3 {
            expected_identity.digest = "abc".to_owned();
        }
        let mut value = bundle(&output);
        value.protected_resources.push(BackendResource {
            path,
            expected_identity,
        });
        assert!(validation_error(&value).message.contains(message));
    }
}

#[cfg(unix)]
#[test]
fn resources_reject_non_utf8_and_unreadable_files() {
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let mut value = bundle(&temp.path().join("output.srt"));
    value.protected_resources.push(BackendResource {
        path: temp.path().join(std::ffi::OsString::from_vec(vec![0xff])),
        expected_identity: fake_identity('a'),
    });
    assert!(validation_error(&value).message.contains("valid UTF-8"));

    let unreadable = temp.path().join("unreadable.bin");
    std::fs::write(&unreadable, b"value").unwrap();
    let expected = identity(&unreadable);
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o0)).unwrap();
    value.protected_resources[0] = BackendResource {
        path: unreadable.clone(),
        expected_identity: expected,
    };
    let result = validate(&value);
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o600)).unwrap();
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("unreadable resource must fail"),
    };
    assert!(error.message.contains("cannot hash protected resource"));
}

#[test]
fn canonical_aliases_reject_matching_and_conflicting_identities() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("input.bin");
    std::fs::write(&path, b"value").unwrap();
    let alternate = temp.path().join(".").join("input.bin");
    for second_identity in [identity(&path), fake_identity('b')] {
        let mut value = bundle(&temp.path().join("output.srt"));
        value.protected_resources = vec![
            BackendResource {
                path: alternate.clone(),
                expected_identity: identity(&path),
            },
            BackendResource {
                path: path.clone(),
                expected_identity: second_identity,
            },
        ];
        value
            .protected_resources
            .sort_by_key(|item| item.path.to_string_lossy().into_owned());
        let message = validation_error(&value).message;
        assert!(message.contains("alias") || message.contains("unique and sorted"));
    }
}

fn validation_error(value: &crate::executor::model::RuntimeBundle) -> crate::RuntimeError {
    match validate(value) {
        Err(error) => error,
        Ok(_) => panic!("invalid resource bundle must fail"),
    }
}

fn validate(
    value: &crate::executor::model::RuntimeBundle,
) -> Result<resource::ValidatedResources, crate::RuntimeError> {
    resource::validate_while(value, &mut || true)
}
