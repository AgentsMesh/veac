use std::collections::BTreeSet;

use super::super::{language_spec, CanonicalRole, ControlUse, GrammarPosition};
use super::legacy_surface::{assert_control_absent, assert_control_contract};

mod audio;
mod color;
mod generator;
mod modifier;

fn assert_group(controls: &[ControlUse], position: GrammarPosition, role: CanonicalRole) {
    let vocabulary = language_spec().vocabulary;
    for control in controls {
        assert_control_contract(*control, position, role);
        assert_control_absent(&vocabulary, *control);
    }
}

fn assert_complete(expected: &[ControlUse], actual: &[ControlUse]) {
    assert_eq!(
        expected.iter().copied().collect::<BTreeSet<_>>(),
        actual.iter().copied().collect()
    );
}
