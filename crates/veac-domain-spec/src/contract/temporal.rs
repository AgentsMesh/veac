#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemporalLoweringOpcode {
    ComposeVector,
    ComposePoint,
    ComposeRect,
}
