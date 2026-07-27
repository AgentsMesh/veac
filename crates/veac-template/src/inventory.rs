use veac_ir::{ClipSource, Frame, ItemId, MaterialId, ProjectEnvelope, SlotConstraint, TrackId};

#[derive(Debug)]
pub(crate) struct Inventory {
    pub slots: Vec<MediaSlot>,
    pub texts: Vec<EditableText>,
}

#[derive(Debug)]
pub(crate) struct MediaSlot {
    pub clip_id: ItemId,
    pub material_id: MaterialId,
    pub constraint: SlotConstraint,
    pub duration: veac_ir::RationalTime,
    pub frame: Option<Frame>,
    pub canvas: (u32, u32),
    pub track_id: TrackId,
    pub track_locked: bool,
    pub source: ClipSource,
}

#[derive(Debug)]
pub(crate) struct EditableText {
    pub clip_id: ItemId,
    pub track_id: TrackId,
    pub track_locked: bool,
    pub source: ClipSource,
}

pub(crate) fn collect(project: &ProjectEnvelope) -> Inventory {
    let mut inventory = Inventory {
        slots: Vec::new(),
        texts: Vec::new(),
    };
    for sequence in &project.project.sequences {
        let canvas = (sequence.settings.width, sequence.settings.height);
        for track in &sequence.tracks {
            for clip in &track.clips {
                if let (Some(constraint), ClipSource::Media { material_id }) =
                    (&clip.replaceable, &clip.source)
                {
                    inventory.slots.push(MediaSlot {
                        clip_id: clip.id.clone(),
                        material_id: material_id.clone(),
                        constraint: constraint.clone(),
                        duration: clip.record_range.duration,
                        frame: clip.visual.as_ref().and_then(|visual| visual.frame),
                        canvas,
                        track_id: track.id.clone(),
                        track_locked: track.state.locked,
                        source: clip.source.clone(),
                    });
                }
                if clip.template_editable_text {
                    inventory.texts.push(EditableText {
                        clip_id: clip.id.clone(),
                        track_id: track.id.clone(),
                        track_locked: track.state.locked,
                        source: clip.source.clone(),
                    });
                }
            }
        }
    }
    inventory
}
