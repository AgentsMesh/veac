use super::*;

#[test]
fn canonical_structure_budget_runs_on_authored_objects() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    for index in sequence.tracks.len()..=MAX_TOTAL_TRACKS as usize {
        sequence.tracks.push(Track {
            id: TrackId::new(format!("trk_budget_{index}")).unwrap(),
            kind: TrackKind::Visual,
            order: index as i32,
            placement_mode: PlacementMode::Free,
            state: TrackState {
                enabled: true,
                muted: false,
                solo: false,
                locked: false,
            },
            routing: TrackRouting::Default,
            clips: vec![],
        });
    }

    assert_code(&validation_codes(&project), "BUDGET_TRACKS");
}
