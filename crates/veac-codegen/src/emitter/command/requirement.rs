use veac_plan::canonical::DeliverableId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendCapabilityKind {
    Encoder,
    Decoder,
    Muxer,
    Demuxer,
    Filter,
    HardwareBackend,
    HardwareDevice,
}

impl BackendCapabilityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Encoder => "encoder",
            Self::Decoder => "decoder",
            Self::Muxer => "muxer",
            Self::Demuxer => "demuxer",
            Self::Filter => "filter",
            Self::HardwareBackend => "hardware backend",
            Self::HardwareDevice => "hardware device",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendRequirement {
    Encoder {
        deliverable_id: DeliverableId,
        name: String,
    },
    Decoder {
        deliverable_id: DeliverableId,
        name: String,
    },
    Muxer {
        deliverable_id: DeliverableId,
        name: String,
    },
    Demuxer {
        deliverable_id: DeliverableId,
        name: String,
    },
    Filter {
        deliverable_id: DeliverableId,
        name: String,
    },
    HardwareBackend {
        deliverable_id: DeliverableId,
        name: String,
    },
    HardwareDevice {
        deliverable_id: DeliverableId,
        name: String,
    },
}

impl BackendRequirement {
    pub fn kind(&self) -> BackendCapabilityKind {
        match self {
            Self::Encoder { .. } => BackendCapabilityKind::Encoder,
            Self::Decoder { .. } => BackendCapabilityKind::Decoder,
            Self::Muxer { .. } => BackendCapabilityKind::Muxer,
            Self::Demuxer { .. } => BackendCapabilityKind::Demuxer,
            Self::Filter { .. } => BackendCapabilityKind::Filter,
            Self::HardwareBackend { .. } => BackendCapabilityKind::HardwareBackend,
            Self::HardwareDevice { .. } => BackendCapabilityKind::HardwareDevice,
        }
    }

    pub fn deliverable_id(&self) -> &DeliverableId {
        match self {
            Self::Encoder { deliverable_id, .. }
            | Self::Decoder { deliverable_id, .. }
            | Self::Muxer { deliverable_id, .. }
            | Self::Demuxer { deliverable_id, .. }
            | Self::Filter { deliverable_id, .. }
            | Self::HardwareBackend { deliverable_id, .. }
            | Self::HardwareDevice { deliverable_id, .. } => deliverable_id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Encoder { name, .. }
            | Self::Decoder { name, .. }
            | Self::Muxer { name, .. }
            | Self::Demuxer { name, .. }
            | Self::Filter { name, .. }
            | Self::HardwareBackend { name, .. }
            | Self::HardwareDevice { name, .. } => name,
        }
    }
}
