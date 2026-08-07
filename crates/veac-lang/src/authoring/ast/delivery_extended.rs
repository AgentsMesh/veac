mod hls;

pub use hls::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFileFormat {
    Mp3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimatedImageFormat {
    Gif,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdaptivePackageFormat {
    Hls,
}
