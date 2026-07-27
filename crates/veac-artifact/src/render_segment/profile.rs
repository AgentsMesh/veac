use serde::Serialize;
use veac_ir::{Deliverable, DeliverableKind, Rational, VideoDeliverable};
use veac_plan::ResolvedRenderPlan;

use crate::{ArtifactResult, ContentDigest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FullRenderSegmentMediaProfile {
    width: u32,
    height: u32,
    frame_rate: Rational,
    deliverable: Deliverable,
}

impl FullRenderSegmentMediaProfile {
    pub(super) fn new(plan: &ResolvedRenderPlan, deliverable: &Deliverable) -> Self {
        Self {
            width: plan.output.width,
            height: plan.output.height,
            frame_rate: plan.output.frame_rate,
            deliverable: deliverable.clone(),
        }
    }

    pub(super) fn digest(&self) -> ArtifactResult<ContentDigest> {
        serde_json_canonicalizer::to_vec(self)
            .map(ContentDigest::sha256)
            .map_err(super::serialization_error)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn frame_rate(&self) -> Rational {
        self.frame_rate
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
