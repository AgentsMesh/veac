use super::{
    ExpressionSite, SourceAudioProcessorField, SourceAudioProcessorKind,
    SourceDeliveryArtifactKind, SourceDeliveryField, SourceNodeKind, SourceNodePath, SourceNodeRef,
    SourcePresetKind,
};

impl ExpressionSite {
    pub fn accepts(&self, kind: SourceNodeKind) -> bool {
        matches!(
            (self, kind),
            (Self::ConstantValue, SourceNodeKind::Constant)
                | (
                    Self::ComponentParameterDefault { .. },
                    SourceNodeKind::Component
                )
                | (
                    Self::ComponentInstanceArgument { .. },
                    SourceNodeKind::ComponentInstance
                )
                | (
                    Self::ComponentLocalInstanceArgument { .. },
                    SourceNodeKind::ComponentLocalInstance
                )
                | (
                    Self::ItemRecordStart
                        | Self::ItemRecordDuration
                        | Self::ItemEnabled
                        | Self::TextContent,
                    SourceNodeKind::Item | SourceNodeKind::ComponentItem
                )
                | (Self::ResourceLocator, SourceNodeKind::Resource)
                | (
                    Self::ModifierParameter { .. },
                    SourceNodeKind::Modifier
                        | SourceNodeKind::Stage
                        | SourceNodeKind::ComponentModifier
                        | SourceNodeKind::ComponentStage
                        | SourceNodeKind::PresetModifier
                        | SourceNodeKind::PresetStage
                )
                | (Self::PresetTextStyleField { .. }, SourceNodeKind::Preset)
                | (Self::PresetTextLayoutField { .. }, SourceNodeKind::Preset)
                | (Self::PresetColorField { .. }, SourceNodeKind::Preset)
                | (
                    Self::PresetAudioProcessorField { .. },
                    SourceNodeKind::PresetAudioProcessor
                )
                | (
                    Self::PresetAudioEqBandField { .. },
                    SourceNodeKind::PresetAudioEqBand
                )
                | (
                    Self::PresetDeliveryField { .. },
                    SourceNodeKind::PresetDeliveryArtifact
                )
        )
    }

    pub(crate) fn accepts_path(&self, path: &SourceNodePath) -> bool {
        match self {
            Self::PresetTextStyleField { .. } => matches!(
                path,
                SourceNodePath::Preset {
                    preset_kind: SourcePresetKind::TextStyle,
                    ..
                }
            ),
            Self::PresetTextLayoutField { .. } => matches!(
                path,
                SourceNodePath::Preset {
                    preset_kind: SourcePresetKind::TextLayout,
                    ..
                }
            ),
            Self::PresetColorField { .. } => matches!(
                path,
                SourceNodePath::Preset {
                    preset_kind: SourcePresetKind::ColorPipeline,
                    ..
                }
            ),
            Self::PresetAudioProcessorField {
                processor_kind,
                field,
            } => {
                matches!(path, SourceNodePath::PresetAudioProcessor { .. })
                    && field.accepts(*processor_kind)
            }
            Self::PresetAudioEqBandField { .. } => {
                matches!(path, SourceNodePath::PresetAudioEqBand { .. })
            }
            Self::PresetDeliveryField { field } => match path {
                SourceNodePath::PresetDeliveryArtifact { artifact_kind, .. } => {
                    field.accepts(*artifact_kind)
                }
                _ => false,
            },
            _ => self.accepts(path.kind()),
        }
    }

    pub fn accepts_target(&self, target: &SourceNodeRef) -> bool {
        self.accepts_path(&target.path)
    }

    pub(crate) fn name(&self) -> Option<&str> {
        match self {
            Self::ComponentParameterDefault { parameter }
            | Self::ComponentInstanceArgument { parameter }
            | Self::ComponentLocalInstanceArgument { parameter }
            | Self::ModifierParameter { parameter } => Some(parameter),
            _ => None,
        }
    }
}

impl SourceAudioProcessorField {
    fn accepts(self, processor: SourceAudioProcessorKind) -> bool {
        use SourceAudioProcessorField as Field;
        use SourceAudioProcessorKind as Kind;
        matches!(
            (self, processor),
            (
                Field::Frequency | Field::Q | Field::Poles,
                Kind::HighPass | Kind::LowPass
            ) | (
                Field::Threshold
                    | Field::Ratio
                    | Field::Attack
                    | Field::Release
                    | Field::Knee
                    | Field::MakeupGain
                    | Field::Mix,
                Kind::Compressor
            ) | (
                Field::Ceiling | Field::Attack | Field::Release,
                Kind::Limiter
            ) | (
                Field::Threshold | Field::Ratio | Field::Attack | Field::Release | Field::Range,
                Kind::Gate
            ) | (
                Field::Integrated | Field::TruePeak | Field::Range,
                Kind::Loudness
            )
        )
    }
}

impl SourceDeliveryField {
    fn accepts(self, artifact: SourceDeliveryArtifactKind) -> bool {
        matches!(self, Self::Target)
            || matches!(
                artifact,
                SourceDeliveryArtifactKind::AudioStem | SourceDeliveryArtifactKind::AudioFile
            ) && matches!(
                self,
                Self::SampleFormat | Self::SampleRate | Self::ChannelLayout
            )
    }
}
