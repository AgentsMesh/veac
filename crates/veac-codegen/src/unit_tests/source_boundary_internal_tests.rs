use super::*;
use crate::unit_tests::emitter_tests::support::{fixture, resolved, time};
use veac_plan::canonical::{PlaybackDirection, Rational, RationalTime, TimeRange};
use veac_plan::ResolvedSourceTimeMap;

#[path = "source_boundary_internal_tests/padding.rs"]
mod padding;
#[path = "source_boundary_internal_tests/required.rs"]
mod required_tests;

fn mapping() -> (veac_plan::ResolvedClip, ResolvedSourceMapping) {
    let plan = resolved(&fixture());
    let clip = plan.sequences[0].tracks[0].clips[0].clone();
    (clip.clone(), clip.source_mapping.expect("source mapping"))
}

fn linear(mapping: &mut ResolvedSourceMapping, start: RationalTime, duration: RationalTime) {
    mapping.time_map = ResolvedSourceTimeMap::Linear {
        source_range_per_repeat: TimeRange { start, duration },
        rate: Rational::new(1, 1).expect("identity rate"),
        repeat: 1,
        direction: PlaybackDirection::Forward,
    };
}

fn required_error(result: Result<Option<Padding>, CodegenErrors>) -> CodegenErrors {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected source-boundary rejection"),
    }
}
