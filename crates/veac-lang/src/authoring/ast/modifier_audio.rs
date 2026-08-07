super::impl_syntax_tokens!(veac_ir::PitchPolicy,
    veac_ir::PitchPolicy::Preserve => "preserve",
    veac_ir::PitchPolicy::FollowSpeed => "follow-speed",
);

super::impl_syntax_tokens!(veac_ir::AudioFadeCurve,
    veac_ir::AudioFadeCurve::Linear => "linear",
    veac_ir::AudioFadeCurve::EqualPower => "equal-power",
    veac_ir::AudioFadeCurve::Exponential => "exponential",
);
