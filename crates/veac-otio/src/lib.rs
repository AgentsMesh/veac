//! Loss-aware OpenTimelineIO interchange for canonical VEAC projects.

mod canonical;
mod error;
mod export;
mod import;
mod loss;
mod model;
mod proposal;
mod time;

pub use canonical::*;
pub use error::*;
pub use export::*;
pub use import::*;
pub use loss::*;
pub use model::*;
pub use proposal::*;

pub const OTIO_TIMELINE_SCHEMA: &str = "Timeline.1";
pub const OTIO_STACK_SCHEMA: &str = "Stack.1";
pub const OTIO_TRACK_SCHEMA: &str = "Track.1";
pub const OTIO_CLIP_SCHEMA: &str = "Clip.2";
pub const OTIO_GAP_SCHEMA: &str = "Gap.1";
pub const OTIO_TIME_RANGE_SCHEMA: &str = "TimeRange.1";
pub const OTIO_RATIONAL_TIME_SCHEMA: &str = "RationalTime.1";

#[cfg(test)]
mod unit_tests;
