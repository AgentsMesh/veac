mod alpha;
mod composite;
mod diff;
mod frame;
mod mask;
mod motion;
mod reveal;

pub use alpha::{alpha_stats, AlphaStats};
pub use composite::{compare_composite, source_over, CompositeStats};
pub use diff::{diff_stats, DiffStats};
pub use frame::{resolve_region, PixelRect};
pub use mask::{bounds, mask, Bounds, Mask};
pub use motion::{motion_deceleration, MotionStats};
pub use reveal::{reveal_prefix, RevealStats};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricError {
    InvalidFrame(String),
    MissingAlpha,
    GeometryMismatch,
    RegionOutsideFrame,
    EmptySelection,
    InvalidThreshold,
}

impl std::fmt::Display for MetricError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFrame(value) => write!(formatter, "invalid frame: {value}"),
            Self::MissingAlpha => write!(formatter, "frame has no alpha channel"),
            Self::GeometryMismatch => write!(formatter, "frame geometries do not match"),
            Self::RegionOutsideFrame => write!(formatter, "region is outside the frame"),
            Self::EmptySelection => write!(formatter, "pixel selection is empty"),
            Self::InvalidThreshold => write!(formatter, "metric threshold is invalid"),
        }
    }
}

impl std::error::Error for MetricError {}
