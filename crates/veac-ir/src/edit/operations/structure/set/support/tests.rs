use std::cell::Cell;

use crate::test_support::sample_project;

use super::*;

#[test]
fn structure_lookup_helpers_find_each_owner_and_report_missing_ids() {
    let mut project = sample_project().project;
    let material = project.materials[0].id.clone();
    let sequence = project.sequences[0].id.clone();
    let track = project.sequences[0].tracks[0].id.clone();
    assert_eq!(material_index(&project, &material).unwrap(), 0);
    assert_eq!(sequence_mut(&mut project, &sequence).unwrap().id, sequence);
    assert_eq!(track_mut(&mut project, &track).unwrap().id, track);
    assert_eq!(unlocked_track(&mut project, &track).unwrap().id, track);

    assert!(material_index(&project, &MaterialId::new("med_absent").unwrap()).is_err());
    assert!(sequence_mut(&mut project, &SequenceId::new("seq_absent").unwrap()).is_err());
    assert!(track_mut(&mut project, &TrackId::new("trk_absent").unwrap()).is_err());

    project.sequences[0].tracks[0].state.locked = true;
    assert!(unlocked_track(&mut project, &track).is_err());
}

#[test]
fn mark_if_marks_only_real_replacements() {
    let marks = Cell::new(0);
    let mut value = 7_u32;
    mark_if(&mut value, 7, || marks.set(marks.get() + 1));
    assert_eq!(value, 7);
    assert_eq!(marks.get(), 0);
    mark_if(&mut value, 9, || marks.set(marks.get() + 1));
    assert_eq!(value, 9);
    assert_eq!(marks.get(), 1);
}
