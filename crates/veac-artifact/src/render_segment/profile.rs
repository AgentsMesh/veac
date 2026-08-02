use serde::Serialize;
use veac_ir::{Deliverable, DeliverableKind, RasterSettings, Rational, VideoDeliverable};

use crate::{ArtifactResult, ContentDigest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FullRenderSegmentMediaProfile {
    raster: RasterSettings,
    deliverable: Deliverable,
}

impl FullRenderSegmentMediaProfile {
    pub(super) fn new(raster: &RasterSettings, deliverable: &Deliverable) -> Self {
        Self {
            raster: raster.clone(),
            deliverable: deliverable.clone(),
        }
    }

    pub(super) fn digest(&self) -> ArtifactResult<ContentDigest> {
        serde_json_canonicalizer::to_vec(self)
            .map(ContentDigest::sha256)
            .map_err(super::serialization_error)
    }

    pub fn width(&self) -> u32 {
        self.raster.width
    }

    pub fn height(&self) -> u32 {
        self.raster.height
    }

    pub fn frame_rate(&self) -> Rational {
        self.raster.frame_rate
    }

    pub fn deliverable(&self) -> &Deliverable {
        &self.deliverable
    }

    pub fn video(&self) -> &VideoDeliverable {
        let DeliverableKind::Video(video) = &self.deliverable.kind else {
            unreachable!("full render-segment profiles are always video")
        };
        video
    }
}
