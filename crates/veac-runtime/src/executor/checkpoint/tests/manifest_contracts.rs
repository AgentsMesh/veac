use super::super::manifest::{self, CheckpointManifest};
use super::support::entry;

#[test]
fn manifest_round_trip_is_strict_canonical_and_sorted() {
    let manifest = CheckpointManifest {
        schema_version: 2,
        outputs: vec![entry("a.srt"), entry("b.srt")],
    };
    let bytes = manifest::encode(&manifest).unwrap();
    assert_eq!(manifest::decode(&bytes).unwrap(), manifest);

    let mut padded = bytes.clone();
    padded.push(b'\n');
    assert!(manifest::decode(&padded)
        .unwrap_err()
        .message
        .contains("strict canonical"));

    let wrong_version = CheckpointManifest {
        schema_version: 1,
        ..manifest.clone()
    };
    assert!(manifest::decode(&manifest::encode(&wrong_version).unwrap()).is_err());

    for outputs in [vec![], vec![entry("b.srt"), entry("a.srt")]] {
        let invalid = CheckpointManifest {
            schema_version: 2,
            outputs,
        };
        assert!(manifest::decode(&manifest::encode(&invalid).unwrap())
            .unwrap_err()
            .message
            .contains("non-empty, unique, and sorted"));
    }
}

#[test]
fn manifest_decode_distinguishes_utf8_ambiguity_and_shape_errors() {
    assert!(manifest::decode(&[0xff])
        .unwrap_err()
        .message
        .contains("not UTF-8"));
    assert!(
        manifest::decode(br#"{"schema_version":1,"schema_version":1,"outputs":[]}"#)
            .unwrap_err()
            .message
            .contains("ambiguous")
    );
    assert!(manifest::decode(b"{}")
        .unwrap_err()
        .message
        .contains("JSON is invalid"));
}
