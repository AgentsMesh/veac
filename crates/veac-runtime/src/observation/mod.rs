//! Bounded, identity-checked media observations for structured visual evidence.

mod command;
mod limits;
mod model;
mod parse;
mod runtime;
mod source;

pub use limits::ObservationLimits;
pub use model::{
    DecodeObservation, DecodeRequest, DecodedFrame, FramePixelFormat, FrameRequest,
    ObservationSource,
};
pub use runtime::MediaObserver;

#[cfg(test)]
mod tests;
