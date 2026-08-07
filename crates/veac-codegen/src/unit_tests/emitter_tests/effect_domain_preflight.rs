use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{bindings, fixture, resolved, time};

#[test]
fn untrusted_plan_effects_require_their_resolved_stream_domain() {
    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].effects = vec![ResolvedEffect {
        id: EffectId::new("fx_audio_without_stream").unwrap(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        effect: Effect::AudioNormalize { target_lufs: -16.0 },
    }];
    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_EFFECT_DOMAIN_INVALID"));
}
