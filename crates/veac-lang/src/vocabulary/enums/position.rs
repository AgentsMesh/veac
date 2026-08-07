use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::LanguageLayer;

macro_rules! define_grammar_positions {
    (
        surface { $($layer:ident { $($variant:ident),+ $(,)? })+ }
        legacy $legacy_layer:ident { $($legacy_variant:ident),+ $(,)? }
    ) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
            Serialize, Deserialize, JsonSchema,
        )]
        #[serde(rename_all = "snake_case")]
        pub enum GrammarPosition {
            $($( $variant, )+)+
            $(#[schemars(skip)] $legacy_variant,)+
        }

        impl GrammarPosition {
            pub const ALL: &'static [Self] = &[$($(Self::$variant,)+)+];

            pub const fn layer(self) -> LanguageLayer {
                match self {
                    $($(Self::$variant)|+ => LanguageLayer::$layer,)+
                    $(Self::$legacy_variant)|+ => LanguageLayer::$legacy_layer,
                }
            }

            pub const fn is_public(self) -> bool {
                self.layer().is_public()
            }
        }
    };
}

define_grammar_positions! {
    surface {
        ExecutableExpression {
            ExpressionBooleanLiteral, ExpressionFunctionCallee, ExpressionNumericUnit,
            ExpressionLetBinding, ExpressionMutableBinding, ExpressionMutableAssignment,
            ExpressionIfBranch, ExpressionElseBranch, ExpressionRangeStep,
            ExpressionClosureIntroducer, ExpressionForBinding, ExpressionForSourceClause,
            ExpressionMatchIntroducer, ExpressionFunctionEffectClause,
            ExpressionFunctionEffectValue, ExpressionTemporalAttachment
        }
        StaticProgram {
            StaticDeclaration, ModuleDeclaration, ModuleMemberModifier, ImportAliasClause,
            MethodReceiver, BuildInputRolePosition, ValueTypePosition, StructuralTypeConstructor,
            TemporalTargetClause, TemporalTargetKind, TemporalSourceClause, TemporalSourceKind,
            TemporalPropertyPosition
        }
    }
    legacy CoreAuthoring {
        CoreBooleanLiteral, CoreTimeUnit, CoreLengthUnit, CorePercentUnit, CoreAngleUnit,
        CoreFrameRateUnit, CoreFrequencyUnit, CoreDecibelUnit, CoreTruePeakUnit,
        CoreLoudnessUnit, CoreTemperatureUnit, CoreExposureUnit, CoreBitRateUnit,
        CoreBitCountUnit, CoreRepetitionUnit,
        ProjectDeclaration, ProjectMember, ProjectEntryReferenceKind, SettingsMember,
        SettingsCanvasSeparator, ResourceKindPosition, ResourceMember,
        ResourceLocatorKindPosition, LocalResourceLocatorMember, RemoteResourceLocatorMember,
        ResourceIdentityKindPosition, ResourceStreamsMember, ResourceStreamSelectionPosition,
        MulticamMember, MulticamSyncBasisPosition, MulticamSyncMember,
        MulticamSyncReferenceKind, MulticamAngleMember, MulticamAngleSourceReferenceKind,
        MulticamSourceMember, MulticamSwitchReferenceKind, MulticamSwitchMember,
        SequenceMember, LayerKindPosition, LayerMember, LayerRouteReferenceKind, ItemMember,
        RecordSpanMember, TrackStateMember, ItemStateMember, MediaSourceReferenceKind,
        SequenceSourceReferenceKind, MulticamSourceReferenceKind, MediaTemplateSlotMember,
        TemplateSlotKindPosition, TemplateMediaKindPosition, TemplateFillPosition,
        SourceKindPosition, TextSourceMember, CaptionSourceMember, TextStyleMember, TextSpanMember,
        TextLayoutMember, TextPathMember, TextAnimationMember, TextHighlightMember,
        TextAnimationTransformMember, TextBackgroundMember, TextOutlineMember,
        ShadowMember, PointMember, VectorMember, RectMember,
        TextFontReferenceKind, TextFontKindPosition, TextFontWeightPosition, TextFontStylePosition,
        TextWrapPosition, TextOverflowPosition, TextHorizontalAlignmentPosition,
        TextVerticalAlignmentPosition, TextWritingModePosition, TextOrientationPosition,
        TextPathAlignmentPosition, TextGranularityPosition,
        ModifierKindPosition, PlacementKindPosition, ModifierAnchorPosition,
        ModifierFitModePosition, MaskShapeKindPosition, PitchPolicyPosition,
        AudioFadeCurvePosition,
        CompositeBlendModePosition, ApplyMixBlendModePosition, HueRangePosition,
        ToneCurveInterpolationPosition, LutInterpolationPosition, ColorStageKindPosition,
        ColorCurveChannelPosition,
        RelationKindPosition, TransitionStyleKindPosition, TransitionAlignmentPosition,
        FadeColorPosition, WipeDirectionPosition, SlideDirectionPosition, ZoomDirectionPosition,
        CircleDirectionPosition, MatteModePosition, TransitionRelationMember,
        TransitionEndpointsMember, RelationItemEndpointReferenceKind, TransitionTimingMember,
        FadeStyleMember,
        WipeStyleMember, SlideStyleMember, ZoomStyleMember, CircleStyleMember,
        PixelizeStyleMember, MatteRelationMember, MatteEndpointsMember, MatteStyleMember,
        SidechainRelationMember, SidechainEndpointsMember, SidechainDynamicsMember,
        SidechainTimingMember, RelationSignalEndpointReferenceKind, GroupRelationMember,
        GroupMemberReferenceKind, AvLinkRelationMember, AvLinkEndpointsMember,
        AvLinkAudioMemberReferenceKind,
        MappingKindPosition, LinearMappingMember, CurveMappingMember, FreezeMappingMember,
        MappingKeyMember, MappingOutOfRangePosition, MappingInterpolationKindPosition,
        ParameterCurveMember,
        ParameterKeyMember, ParameterInterpolationKindPosition, EffectParameterCurveMember,
        EffectParameterKeyMember, EffectParameterInterpolationKindPosition,
        CubicBezierInterpolationMember, SpringInterpolationMember,
        ApplyMember, ApplyScopeKindPosition, ApplyCompositeBandMember,
        ApplyCompositeBandReferenceKind, ApplyItemsMember, ApplyPipelineMember, ApplyMixMember,
        ApplyStageKindPosition,
        LayerPlacementModePosition, TrackPlaybackStatePosition, TrackAudioStatePosition,
        TrackIsolationStatePosition, TrackEditingStatePosition, ItemPlaybackStatePosition,
        DeliveryMember, DeliveryRasterMember, DeliveryRasterCanvasSeparator,
        ArtifactKindPosition, ArtifactTargetKindPosition, ArtifactMember,
        VideoArtifactMember, ImageSequenceArtifactMember, CaptionSidecarArtifactMember,
        AudioStemArtifactMember, ScopeArtifactMember, AudioFileArtifactMember,
        AnimatedImageArtifactMember, StillImageArtifactMember, AdaptivePackageArtifactMember,
        VideoMuxMember, VideoEncodingMember, VideoAudioEncodingMember,
        CrfRateControlMember, AverageRateControlMember, CappedRateControlMember, ColorSpaceMember,
        CaptionSidecarSourceKind, CaptionTracksMember, WavEncodingMember, AudioStemEncodingMember,
        OutputFrameSelectionClause, ScopeCanvasSeparator, Mp3EncodingMember, GifEncodingMember,
        ImageNumberingFromClause, HlsPackageMember, HlsAudioMember, HlsAudioEncodingMember,
        HlsRenditionMember, HlsVideoEncodingMember, HlsCappedRateControlMember,
        HlsRenditionCanvasSeparator,
        GeneratorKindPosition, GradientGeometryKindPosition, ShapeGeometryKindPosition,
        PaintKindPosition, SolidGeneratorMember, ShapeGeneratorMember,
        RectangleGeometryMember, RoundedRectangleGeometryMember, EllipseGeometryMember,
        PolygonGeometryMember, PathGeometryMember, LinearGradientMember, RadialGradientMember,
        LayoutModifierMember, AnchorPlacementMember, AbsolutePlacementMember, FrameMember,
        TransformModifierMember, CompositeModifierMember, SurfaceModifierMember,
        MaskModifierMember, RoundedRectangleMaskMember, PolygonMaskMember, PathMaskMember,
        AudioModifierMember, AudioCrossfadeMember, EqProcessorMember, EqBandMember,
        FilterProcessorMember, CompressorProcessorMember, LimiterProcessorMember,
        GateProcessorMember, LoudnessProcessorMember, EffectModifierMember, ColorModifierMember,
        BasicColorMember, RgbMatrixMember, HslColorMember, ColorCurvesMember, ColorCurveMember,
        ColorWheelsMember, LutColorMember, LutResourceReferenceKind,
        AudioProcessorKindPosition, AnnotationKindPosition, AnnotationMember,
        AnnotationTargetReferenceKind, AnnotationPointSpanMember, AnnotationRangeSpanMember,
        AnnotationProvenanceMember, MarkerPayloadMember, LanguagePayloadMember,
        LanguageCandidateMember, SceneBoundaryPayloadMember, BeatPayloadMember,
        SilencePayloadMember, FillerPayloadMember, HighlightPayloadMember, ReviewPayloadMember,
        AnnotationTimingKindPosition, FillerSuggestionPosition, ReviewActionPosition,
        OutputFormatPosition, VideoCodecPosition, PixelFormatPosition, AlphaModePosition,
        AudioCodecPosition, CaptionOutputPosition, MuxLayoutPosition,
        MuxAudioChannelLayoutPosition, VideoRateControlKindPosition,
        VideoAudioSelectionPosition, HlsAudioSelectionPosition, HlsAudioCodecPosition,
        AudioFileChannelLayoutPosition, HlsAudioChannelLayoutPosition,
        AudioFileFormatPosition, AnimatedImageFormatPosition, GifPlaybackValuePosition,
        AdaptivePackageFormatPosition, HlsVideoCodecPosition, HardwareSelectionPosition,
        VideoGopPosition, VideoBFramesPosition, VideoProfileSelectionPosition,
        VideoLevelPosition, VideoColorSpaceSelectionPosition, HlsProfilePosition,
        HlsLevelPosition, HlsBFramesPosition, HlsColorSpaceSelectionPosition,
        AudioMixSourceKindPosition, PassModePosition, HardwareBackendPosition,
        ImageFormatPosition, CaptionSidecarFormatPosition, AudioStemFormatPosition,
        VideoScopePosition, GifDitherPosition, VideoProfilePosition, ColorPrimariesPosition,
        ColorTransferPosition, ColorMatrixPosition, ColorRangePosition
    }
}
