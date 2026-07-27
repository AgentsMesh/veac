use std::collections::BTreeSet;

use veac_artifact::MediaRole;
use veac_plan::canonical::{CaptionOutput, CaptionSidecarFormat, Deliverable, DeliverableKind};
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
            visual::collect(plan, settings.captions == CaptionOutput::BurnIn, &mut usage);
            if settings.audio.is_some() {
                audio::collect(plan, AudioSelection::Master, &mut usage);
            }
        }
        DeliverableKind::ImageSequence(_) | DeliverableKind::Scope(_) => {
            visual::collect(plan, false, &mut usage);
        }
        DeliverableKind::AudioStem(settings) => {
            audio::collect(plan, AudioSelection::from(&settings.source), &mut usage);
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
    usage.resource(&content.style.font.input_id);
    if fallbacks {
        for font in &content.style.fallback_fonts {
            usage.resource(&font.input_id);
        }
    }
    for font in content
        .style
        .spans
        .iter()
        .filter_map(|span| span.font.as_ref())
    {
        usage.resource(&font.input_id);
    }
}
