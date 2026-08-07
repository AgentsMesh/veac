use crate::vocabulary::{control_uses::audio as c, CanonicalRole as R, GrammarPosition as P};

use super::{assert_complete, assert_group};

#[test]
fn audio_controls_have_exact_owners_and_roles() {
    let groups = [
        (
            &[
                c::MODIFIER_GAIN_FIELD,
                c::MODIFIER_PAN_FIELD,
                c::MODIFIER_MUTED_FIELD,
                c::MODIFIER_NORMALIZE_FIELD,
                c::MODIFIER_PITCH_FIELD,
                c::MODIFIER_CROSSFADE_FIELD,
            ][..],
            P::AudioModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[c::MODIFIER_PROCESSOR_MEMBER],
            P::AudioModifierMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::CROSSFADE_IN_FIELD,
                c::CROSSFADE_OUT_FIELD,
                c::CROSSFADE_CURVE_FIELD,
            ],
            P::AudioCrossfadeMember,
            R::FieldIntroducer,
        ),
        (
            &[c::EQ_BAND_MEMBER],
            P::EqProcessorMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::EQ_BAND_FREQUENCY_FIELD,
                c::EQ_BAND_GAIN_FIELD,
                c::EQ_BAND_Q_FIELD,
            ],
            P::EqBandMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::FILTER_FREQUENCY_FIELD,
                c::FILTER_Q_FIELD,
                c::FILTER_POLES_FIELD,
            ],
            P::FilterProcessorMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::COMPRESSOR_THRESHOLD_FIELD,
                c::COMPRESSOR_RATIO_FIELD,
                c::COMPRESSOR_ATTACK_FIELD,
                c::COMPRESSOR_RELEASE_FIELD,
                c::COMPRESSOR_KNEE_FIELD,
                c::COMPRESSOR_MAKEUP_GAIN_FIELD,
                c::COMPRESSOR_MIX_FIELD,
            ],
            P::CompressorProcessorMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::LIMITER_CEILING_FIELD,
                c::LIMITER_ATTACK_FIELD,
                c::LIMITER_RELEASE_FIELD,
            ],
            P::LimiterProcessorMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::GATE_THRESHOLD_FIELD,
                c::GATE_RATIO_FIELD,
                c::GATE_ATTACK_FIELD,
                c::GATE_RELEASE_FIELD,
                c::GATE_RANGE_FIELD,
            ],
            P::GateProcessorMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::LOUDNESS_INTEGRATED_FIELD,
                c::LOUDNESS_TRUE_PEAK_FIELD,
                c::LOUDNESS_RANGE_FIELD,
            ],
            P::LoudnessProcessorMember,
            R::FieldIntroducer,
        ),
    ];
    let expected = groups
        .iter()
        .flat_map(|(values, _, _)| *values)
        .copied()
        .collect::<Vec<_>>();
    for (controls, position, role) in groups {
        assert_group(controls, position, role);
    }
    assert_complete(&expected, c::ALL);
}

#[test]
fn shared_audio_words_keep_distinct_processor_owners() {
    assert_eq!(
        c::COMPRESSOR_ATTACK_FIELD.word(),
        c::LIMITER_ATTACK_FIELD.word()
    );
    assert_eq!(c::GATE_RANGE_FIELD.word(), c::LOUDNESS_RANGE_FIELD.word());
    assert_ne!(c::COMPRESSOR_ATTACK_FIELD, c::LIMITER_ATTACK_FIELD);
    assert_ne!(c::GATE_RANGE_FIELD, c::LOUDNESS_RANGE_FIELD);
}
