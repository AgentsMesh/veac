use std::collections::BTreeSet;

use crate::authoring::OutputSentinel;

use super::super::VocabularyCategory;
use super::super::{catalog, catalog_core, catalog_output, language_spec, GrammarPosition};
use super::legacy_surface::assert_use_absent;

mod round_trip;

#[test]
fn core_kind_sets_remain_typed_but_are_not_published() {
    assert_eq!(catalog_core::KIND_SETS.len(), 10);
    assert_eq!(
        catalog_core::KIND_SETS
            .iter()
            .map(|set| set.tokens.len())
            .collect::<Vec<_>>(),
        [6, 4, 6, 8, 5, 9, 5, 7, 9, 2]
    );
    assert_eq!(choice_count(&catalog_core::KIND_SETS), 61);
    assert_eq!(context_count(&catalog_core::KIND_SETS), 10);
    round_trip::core_kinds();
    assert_sets_not_published(&catalog_core::KIND_SETS);
}

#[test]
fn output_sets_remain_typed_but_are_not_published() {
    assert_eq!(catalog_output::SETS.len(), 45);
    assert_eq!(choice_count(&catalog_output::SETS), 167);
    assert_eq!(context_count(&catalog_output::SETS), 43);
    for set in &catalog_output::SETS {
        assert!(!set.tokens.is_empty());
        assert_eq!(
            set.tokens.iter().copied().collect::<BTreeSet<_>>().len(),
            set.tokens.len()
        );
    }
    round_trip::outputs();
    assert_sets_not_published(&catalog_output::SETS);
    for (sentinel, position) in OutputSentinel::ALL.into_iter().zip([
        GrammarPosition::VideoAudioSelectionPosition,
        GrammarPosition::HardwareSelectionPosition,
        GrammarPosition::HardwareSelectionPosition,
        GrammarPosition::VideoGopPosition,
        GrammarPosition::VideoColorSpaceSelectionPosition,
    ]) {
        assert!(catalog_output::SETS
            .iter()
            .any(|set| { set.context == position && set.tokens.contains(&sentinel.as_str()) }));
    }
}

#[test]
fn duplicate_legacy_enum_spellings_do_not_leak_owner_positions() {
    let vocabulary = language_spec().vocabulary;
    for (spelling, positions) in [
        (
            "flac",
            &[
                GrammarPosition::AudioCodecPosition,
                GrammarPosition::AudioStemFormatPosition,
            ][..],
        ),
        (
            "aac",
            &[
                GrammarPosition::AudioCodecPosition,
                GrammarPosition::VideoAudioSelectionPosition,
                GrammarPosition::HlsAudioCodecPosition,
            ],
        ),
        (
            "bt709",
            &[
                GrammarPosition::ColorPrimariesPosition,
                GrammarPosition::ColorTransferPosition,
                GrammarPosition::ColorMatrixPosition,
            ],
        ),
        (
            "video",
            &[
                GrammarPosition::ResourceKindPosition,
                GrammarPosition::LayerKindPosition,
                GrammarPosition::ArtifactKindPosition,
            ],
        ),
    ] {
        for position in positions {
            assert_use_absent(
                &vocabulary,
                spelling,
                VocabularyCategory::EnumValue,
                *position,
            );
        }
    }
}

#[test]
fn published_enum_spellings_equal_the_catalog_term_union() {
    let vocabulary = language_spec().vocabulary;
    let cataloged = catalog::TERM_SET_GROUPS
        .iter()
        .flat_map(|sets| sets.iter())
        .filter(|set| set.context.is_public())
        .flat_map(|set| set.tokens.iter().copied())
        .collect::<BTreeSet<_>>();
    let published = vocabulary
        .entries
        .iter()
        .filter(|entry| {
            entry
                .uses
                .iter()
                .any(|usage| usage.category == VocabularyCategory::EnumValue)
        })
        .map(|entry| entry.spelling.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(published, cataloged);
}

fn choice_count(sets: &[super::super::catalog::TermSet]) -> usize {
    sets.iter().map(|set| set.tokens.len()).sum()
}

fn context_count(sets: &[super::super::catalog::TermSet]) -> usize {
    sets.iter()
        .map(|set| set.context)
        .collect::<BTreeSet<_>>()
        .len()
}

fn assert_sets_not_published(sets: &[super::super::catalog::TermSet]) {
    let vocabulary = language_spec().vocabulary;
    for set in sets {
        for token in set.tokens {
            assert_use_absent(
                &vocabulary,
                token,
                VocabularyCategory::EnumValue,
                set.context,
            );
        }
    }
}
