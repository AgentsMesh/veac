use veac_evidence::*;

const ALL_FEATURES: &str = include_str!("fixtures/authored_all_features.veac");

#[test]
fn authored_suite_executes_decodes_validates_and_records_provenance() {
    let authored = build_evidence_source(ALL_FEATURES).unwrap();
    assert_eq!(authored.root_module, "evidence.veac");
    assert_eq!(authored.sources.len(), 1);
    assert_eq!(authored.sources["evidence.veac"], ALL_FEATURES);
    assert!(!authored.sources.contains_key(EVIDENCE_MODULE_ID));
    let inventory = authored
        .source_index
        .inventory(&authored.source_revision())
        .unwrap();
    assert_eq!(inventory.modules.len(), 1);
    assert_eq!(authored.suite.id, "all-features");
    assert_eq!(authored.suite.sources.len(), 3);
    assert_eq!(authored.suite.samples.len(), 6);
    assert_eq!(authored.suite.regions.len(), 2);
    assert_eq!(authored.suite.assertions.len(), 11);
    assert_eq!(
        authored.suite_sha256,
        sha256_hex(&canonical_json(&authored.suite).unwrap())
    );
}

#[test]
fn every_closed_authored_variant_maps_to_the_typed_model() {
    let suite = build_evidence_source(ALL_FEATURES).unwrap().suite;
    assert!(matches!(
        suite.sources[0].binding,
        SourceBinding::Deliverable { .. }
    ));
    assert!(matches!(
        suite.sources[1].binding,
        SourceBinding::Artifact { .. }
    ));
    assert!(matches!(
        suite.sources[2].binding,
        SourceBinding::BoundInput { .. }
    ));
    assert!(matches!(
        suite.regions[0].space,
        RegionSpace::Normalized { .. }
    ));
    assert!(matches!(suite.regions[1].space, RegionSpace::Pixels { .. }));
    let kinds = suite
        .assertions
        .iter()
        .map(AssertionSpec::kind)
        .collect::<Vec<_>>();
    for expected in [
        AssertionKind::DecodeComplete,
        AssertionKind::Alpha,
        AssertionKind::PixelDiff,
        AssertionKind::Bounds,
        AssertionKind::LayerOrder,
        AssertionKind::CompositeOver,
        AssertionKind::RevealOrder,
        AssertionKind::MotionProfile,
    ] {
        assert!(kinds.contains(&expected));
    }
    assert!(suite.assertions.iter().any(|value| matches!(
        value,
        AssertionSpec::Bounds(BoundsSpec {
            mask: MaskSpec::Difference { .. },
            ..
        })
    )));
    assert!(suite.assertions.iter().any(|value| matches!(
        value,
        AssertionSpec::Bounds(BoundsSpec {
            mask: MaskSpec::Luma { .. },
            ..
        })
    )));
    assert!(suite.assertions.iter().any(|value| matches!(
        value,
        AssertionSpec::MotionProfile(MotionProfileSpec {
            metric: MotionMetric::AlphaCentroid { .. },
            ..
        })
    )));
}

#[test]
fn diff_channel_variants_are_closed_and_decoded() {
    for (source_name, expected) in [
        ("Rgb", DiffChannels::Rgb),
        ("Rgba", DiffChannels::Rgba),
        ("Alpha", DiffChannels::Alpha),
    ] {
        let source =
            ALL_FEATURES.replace("DiffChannels.Rgba", &format!("DiffChannels.{source_name}"));
        let suite = build_evidence_source(&source).unwrap().suite;
        let channels = suite.assertions.iter().find_map(|value| match value {
            AssertionSpec::PixelDiff(spec) => Some(spec.channels),
            _ => None,
        });
        assert_eq!(channels, Some(expected));
    }
}

#[test]
fn source_and_suite_revisions_have_independent_identity() {
    let first = build_evidence_source(ALL_FEATURES).unwrap();
    let second = build_evidence_source(&format!("{ALL_FEATURES}\n")).unwrap();
    assert_ne!(first.source_revision, second.source_revision);
    assert_eq!(first.suite_sha256, second.suite_sha256);
}

#[test]
fn authored_contract_schema_and_entry_points_are_stable() {
    assert_eq!(evidence_entry_contract().function(), "evidence");
    assert_eq!(
        evidence_suite_json_schema().unwrap()["title"],
        "EvidenceSuiteV1"
    );
    assert_eq!(
        authored_evidence_source_index_json_schema()["title"],
        "SourceIndexInventory"
    );
}
