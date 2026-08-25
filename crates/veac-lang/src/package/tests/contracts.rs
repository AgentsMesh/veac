use super::*;

#[test]
fn scalar_contracts_accept_only_canonical_exact_values() {
    for valid in ["veac", "acme.motion-2", "a"] {
        assert_eq!(name(valid).as_str(), valid);
        assert_eq!(valid.parse::<PackageName>().unwrap().to_string(), valid);
    }
    for invalid in ["", "Acme", "2d", "a_thing", "a..b", "a-"] {
        assert!(PackageName::new(invalid).is_err(), "accepted {invalid}");
    }
    for valid in ["0.0.0", "1.2.3", "1.0.0-alpha.1", "2.0.0+build-7"] {
        assert_eq!(version(valid).as_str(), valid);
        assert_eq!(valid.parse::<ExactVersion>().unwrap().to_string(), valid);
    }
    for invalid in ["latest", "^1.2", "1.2", "01.2.3", "1.2.3-01", "1.2.3+"] {
        assert!(ExactVersion::new(invalid).is_err(), "accepted {invalid}");
    }
    let sha = digest('a');
    assert_eq!(sha.as_str(), "a".repeat(64));
    assert_eq!(sha.to_string().parse::<Sha256Digest>().unwrap(), sha);
    for invalid in ["a", &"A".repeat(64), &"g".repeat(64)] {
        assert!(Sha256Digest::parse(invalid).is_err());
    }
}

#[test]
fn manifest_json_is_strict_versioned_and_canonical() {
    let manifest = PackageManifestV1::new(
        identity("root", "1.0.0"),
        "main.veac",
        digest('c'),
        vec![dependency("util", "2.0.0")],
        digest('a'),
    );
    let json = canonical_package_manifest_json(&manifest).unwrap();
    assert_eq!(parse_package_manifest_json(&json).unwrap(), manifest);
    assert!(json.starts_with(r#"{"api_sha256":"#));
    assert!(package_manifest_json_schema().unwrap()["additionalProperties"] == false);

    let duplicate = json.replacen(
        r#""entry":"main.veac""#,
        r#""entry":"main.veac","entry":"main.veac""#,
        1,
    );
    assert_eq!(
        parse_package_manifest_json(&duplicate).unwrap_err().kind(),
        PackageErrorKind::Json
    );
    let unknown = json.replacen('{', r#"{"unknown":true,"#, 1);
    assert_eq!(
        parse_package_manifest_json(&unknown).unwrap_err().kind(),
        PackageErrorKind::Json
    );
    let wrong = json.replace(PACKAGE_MANIFEST_SCHEMA, "https://invalid.example/v1");
    assert_eq!(
        parse_package_manifest_json(&wrong).unwrap_err().kind(),
        PackageErrorKind::Contract
    );
    assert_eq!(
        parse_package_manifest_json(&" ".repeat(MAX_PACKAGE_JSON_BYTES + 1))
            .unwrap_err()
            .kind(),
        PackageErrorKind::Contract
    );
}

#[test]
fn lock_requires_sorted_unique_exact_packages_and_canonical_content() {
    let first = locked("alpha", "1.0.0", vec![]);
    let second = locked("beta", "1.0.0", vec![dependency("alpha", "1.0.0")]);
    let lock = package_lock(identity("root", "1.0.0"), vec![first.clone(), second]);
    let json = canonical_package_lock_json(&lock).unwrap();
    assert_eq!(parse_package_lock_json(&json).unwrap(), lock);
    assert!(package_lock_json_schema().unwrap()["additionalProperties"] == false);

    let mut reversed = lock.clone();
    reversed.packages.reverse();
    assert!(reversed.validate().is_err());
    let mut duplicate_path = lock.clone();
    duplicate_path.packages[1].path = first.path;
    assert!(duplicate_path.validate().is_err());
    let mut bad_digest = lock;
    bad_digest.packages[0].content_sha256 = digest('f');
    assert!(bad_digest.validate().is_err());
}

#[test]
fn relative_contract_paths_reject_escape_and_reserved_files() {
    for invalid in ["", ".", "../x", "a/../x", "/tmp/x", "a\\b", "a:b", "a//b"] {
        assert!(
            validation::relative_path(invalid).is_err(),
            "accepted {invalid}"
        );
    }
    assert!(validation::relative_path("vendor/acme").is_ok());
    assert!(validation::source_path("src/lib.veac").is_ok());
    assert!(validation::source_path(PACKAGE_API_FILE).is_err());
}
