use veac_ir::TemporalType;

use crate::{ResolvedClip, ResolvedClipSource};

use super::{Collector, TemporalSinkScope};

impl Collector {
    pub(super) fn clip(&mut self, clip: &ResolvedClip, base: &str, scope: &TemporalSinkScope) {
        if let Some(visual) = &clip.visual {
            self.visual(visual, &format!("{base}/visual"), scope);
        }
        if let Some(audio) = &clip.audio {
            self.leaf(
                &audio.gain,
                TemporalType::Scalar,
                &format!("{base}/audio/gain"),
                scope,
            );
            self.leaf(
                &audio.pan,
                TemporalType::Scalar,
                &format!("{base}/audio/pan"),
                scope,
            );
        }
        if let ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } =
            &clip.source
        {
            if let Some(animation) = content.styled().and_then(|style| style.animation.as_ref()) {
                self.text(
                    animation,
                    &format!("{base}/source/content/animation"),
                    scope,
                );
            }
        }
        self.effects(&clip.effects, &format!("{base}/effects"), scope);
    }
}
