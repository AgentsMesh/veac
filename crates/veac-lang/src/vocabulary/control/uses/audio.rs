use super::super::use_macro::define_control_uses;

define_control_uses! {
    MODIFIER_GAIN_FIELD => "gain" @ AudioModifierMember : FieldIntroducer;
    MODIFIER_PAN_FIELD => "pan" @ AudioModifierMember : FieldIntroducer;
    MODIFIER_MUTED_FIELD => "muted" @ AudioModifierMember : FieldIntroducer;
    MODIFIER_NORMALIZE_FIELD => "normalize" @ AudioModifierMember : FieldIntroducer;
    MODIFIER_PITCH_FIELD => "pitch" @ AudioModifierMember : FieldIntroducer;
    MODIFIER_PROCESSOR_MEMBER => "processor" @ AudioModifierMember : DeclarationIntroducer;
    MODIFIER_CROSSFADE_FIELD => "crossfade" @ AudioModifierMember : FieldIntroducer;
    CROSSFADE_IN_FIELD => "fade-in" @ AudioCrossfadeMember : FieldIntroducer;
    CROSSFADE_OUT_FIELD => "fade-out" @ AudioCrossfadeMember : FieldIntroducer;
    CROSSFADE_CURVE_FIELD => "curve" @ AudioCrossfadeMember : FieldIntroducer;
    EQ_BAND_MEMBER => "band" @ EqProcessorMember : DeclarationIntroducer;
    EQ_BAND_FREQUENCY_FIELD => "frequency" @ EqBandMember : FieldIntroducer;
    EQ_BAND_GAIN_FIELD => "gain" @ EqBandMember : FieldIntroducer;
    EQ_BAND_Q_FIELD => "q" @ EqBandMember : FieldIntroducer;
    FILTER_FREQUENCY_FIELD => "frequency" @ FilterProcessorMember : FieldIntroducer;
    FILTER_Q_FIELD => "q" @ FilterProcessorMember : FieldIntroducer;
    FILTER_POLES_FIELD => "poles" @ FilterProcessorMember : FieldIntroducer;
    COMPRESSOR_THRESHOLD_FIELD => "threshold" @ CompressorProcessorMember : FieldIntroducer;
    COMPRESSOR_RATIO_FIELD => "ratio" @ CompressorProcessorMember : FieldIntroducer;
    COMPRESSOR_ATTACK_FIELD => "attack" @ CompressorProcessorMember : FieldIntroducer;
    COMPRESSOR_RELEASE_FIELD => "release" @ CompressorProcessorMember : FieldIntroducer;
    COMPRESSOR_KNEE_FIELD => "knee" @ CompressorProcessorMember : FieldIntroducer;
    COMPRESSOR_MAKEUP_GAIN_FIELD => "makeup-gain" @ CompressorProcessorMember : FieldIntroducer;
    COMPRESSOR_MIX_FIELD => "mix" @ CompressorProcessorMember : FieldIntroducer;
    LIMITER_CEILING_FIELD => "ceiling" @ LimiterProcessorMember : FieldIntroducer;
    LIMITER_ATTACK_FIELD => "attack" @ LimiterProcessorMember : FieldIntroducer;
    LIMITER_RELEASE_FIELD => "release" @ LimiterProcessorMember : FieldIntroducer;
    GATE_THRESHOLD_FIELD => "threshold" @ GateProcessorMember : FieldIntroducer;
    GATE_RATIO_FIELD => "ratio" @ GateProcessorMember : FieldIntroducer;
    GATE_ATTACK_FIELD => "attack" @ GateProcessorMember : FieldIntroducer;
    GATE_RELEASE_FIELD => "release" @ GateProcessorMember : FieldIntroducer;
    GATE_RANGE_FIELD => "range" @ GateProcessorMember : FieldIntroducer;
    LOUDNESS_INTEGRATED_FIELD => "integrated" @ LoudnessProcessorMember : FieldIntroducer;
    LOUDNESS_TRUE_PEAK_FIELD => "true-peak" @ LoudnessProcessorMember : FieldIntroducer;
    LOUDNESS_RANGE_FIELD => "range" @ LoudnessProcessorMember : FieldIntroducer;
}
