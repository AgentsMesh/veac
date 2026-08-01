use std::collections::BTreeSet;

use veac_artifact::MediaRole;
use veac_plan::canonical::{
    AdaptivePackage, CaptionOutput, CaptionSidecarFormat, Deliverable, DeliverableKind,
};
use veac_plan::{PlanInputId, ResolvedRenderPlan, ResolvedText};

mod audio;
mod visual;

pub(in crate::emitter) use audio::{visit as visit_audio, Selection as AudioSelection};

#[derive(Default)]
pub(in crate::emitter) struct Usage {
    media: BTreeSet<(PlanInputId, MediaRole)>,
    resources: BTreeSet<PlanInputId>,
}

pub(in crate::emitter) fn required(plan: &ResolvedRenderPlan, deliverable: &Deliverable) -> Usage {
    let mut usage = Usage::default();
    match &deliverable.kind {
        DeliverableKind::Video(settings) => {
            visual::collect(plan, burn_captions(plan), &mut usage);
            if settings.audio.is_some() {
                audio::collect(plan, AudioSelection::Master, &mut usage);
            }
        }
        DeliverableKind::ImageSequence(_) | DeliverableKind::Scope(_) => {
            visual::collect(plan, burn_captions(plan), &mut usage);
        }
        DeliverableKind::AudioStem(settings) => {
            audio::collect(plan, AudioSelection::from(&settings.source), &mut usage);
        }
        DeliverableKind::AudioFile(settings) => {
            audio::collect(plan, AudioSelection::from(&settings.source), &mut usage);
        }
        DeliverableKind::AnimatedImage(_) | DeliverableKind::StillImage(_) => {
            visual::collect(plan, burn_captions(plan), &mut usage);
        }
        DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) => {
            visual::collect(plan, burn_captions(plan), &mut usage);
            if let Some(audio) = &settings.audio {
                audio::collect(plan, AudioSelection::from(&audio.source), &mut usage);
            }
        }
        DeliverableKind::CaptionSidecar(settings)
            if settings.format == CaptionSidecarFormat::Ass =>
        {
            let Some(sequence) = plan
                .sequences
                .iter()
                .find(|value| value.id == plan.entry_sequence_id)
            else {
                return usage;
            };
            for track in sequence
                .tracks
                .iter()
                .filter(|track| settings.track_ids.contains(&track.id))
            {
                for clip in &track.clips {
                    if let veac_plan::ResolvedClipSource::Caption { content, .. } = &clip.source {
                        text_resources(content, false, &mut usage);
                    }
                }
            }
        }
        DeliverableKind::CaptionSidecar(_) => {}
    }
    usage
}

fn burn_captions(plan: &ResolvedRenderPlan) -> bool {
    plan.output
        .raster
        .as_ref()
        .is_some_and(|raster| raster.captions == CaptionOutput::BurnIn)
}

impl Usage {
    pub(super) fn contains(&self, id: &PlanInputId, role: MediaRole) -> bool {
        self.media.contains(&(id.clone(), role))
    }

    pub(in crate::emitter) fn resources(&self) -> &BTreeSet<PlanInputId> {
        &self.resources
    }

    fn media(&mut self, id: &PlanInputId, role: MediaRole) {
        self.media.insert((id.clone(), role));
    }

    fn resource(&mut self, id: &PlanInputId) {
        self.resources.insert(id.clone());
    }
}

fn text_resources(content: &ResolvedText, fallbacks: bool, usage: &mut Usage) {
    let Some(style) = content.styled() else {
        return;
    };
    usage.resource(&style.font.input_id);
    if fallbacks {
        for font in &style.fallback_fonts {
            usage.resource(&font.input_id);
        }
    }
    for font in style.spans.iter().filter_map(|span| span.font.as_ref()) {
        usage.resource(&font.input_id);
    }
}
