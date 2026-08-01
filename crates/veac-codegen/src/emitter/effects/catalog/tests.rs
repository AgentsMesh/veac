use std::collections::BTreeSet;

use super::*;

#[test]
fn backend_effect_catalog_exactly_matches_the_canonical_registry() {
    let registry: BTreeSet<_> = veac_plan::canonical::built_in_effects()
        .iter()
        .map(|effect| effect.effect_type)
        .collect();
    let backend: BTreeSet<_> = EFFECTS.iter().map(|(name, _)| *name).collect();
    assert_eq!(backend.len(), EFFECTS.len(), "duplicate backend effect key");
    assert_eq!(backend, registry);
    for (name, kind) in EFFECTS {
        assert_eq!(effect_kind(name), Some(*kind));
    }
    assert_eq!(effect_kind("video.unknown"), None);
}

#[test]
fn backend_parameter_kinds_exactly_match_the_canonical_union() {
    let canonical: BTreeSet<_> = ParameterValueKind::ALL.into_iter().collect();
    let backend: BTreeSet<_> = PARAMETER_KINDS.iter().copied().collect();
    assert_eq!(backend.len(), PARAMETER_KINDS.len());
    assert_eq!(backend, canonical);
}
