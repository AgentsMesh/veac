mod clips;
mod materials;
mod multicam;
mod project;
mod relations;
mod visual;

pub(crate) use clips::media_clip;
pub(crate) use multicam::multicam_project;
pub(crate) use project::{empty_temporal, executable_manifest, linked_project, sample_project};
pub(crate) use relations::{add_transition, transition_from, transition_mut};
pub(crate) use visual::{identity_layout_visual, visual};

use crate::{RationalTime, TimeRange};

pub(crate) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}

pub(crate) fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}
