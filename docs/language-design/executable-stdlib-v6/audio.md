# Audio Processing And Routing

The authoritative machine source for this family is
[`spec/domain/audio`](../../../spec/domain/audio). The declarations below are its checked
documentation projection; `scripts/stdlib_codegen.py` rejects any type or signature drift.

## DomainTypes

```text
AudioStyle, AudioPlayback, PitchPolicy
AudioProcessor, ParametricEqBand
CompressorSettings, LimiterSettings, GateSettings, LoudnessTarget
AudioCrossfadeChoice, AudioFadeCurve
AudioBus
```

Gain is a linear amplitude factor. Pan is a scalar in `[-1, 1]`, from left to right. Values whose names
end in `_hz`, `_db`, `_ms`, `_lufs`, `_dbtp`, or `_lu` carry that unit in the operation contract even
though the current primitive representation is `scalar`.

## Playback And Routing

```veac
pitch_preserve() -> PitchPolicy
pitch_follow_speed() -> PitchPolicy
audio_playback(muted: bool, normalize: bool, pitch: PitchPolicy) -> AudioPlayback
audio_bus(key: identifier) -> AudioBus
track_routing_default() -> TrackRouting
track_routing_bus(bus: AudioBus) -> TrackRouting
```

`AudioBus` is an opaque stable bus reference, not free text. The two routing signatures are the same
operations cataloged with timeline settings. Preserve uses time stretching; follow-speed resamples.

## Equalization And Filters

```veac
parametric_eq_band(key: identifier, frequency_hz: scalar, gain_db: scalar,
                   q: scalar) -> ParametricEqBand
audio_parametric_eq(key: identifier, bands: list<ParametricEqBand>) -> AudioProcessor
audio_high_pass(key: identifier, frequency_hz: scalar, q: scalar,
                poles: int) -> AudioProcessor
audio_low_pass(key: identifier, frequency_hz: scalar, q: scalar,
               poles: int) -> AudioProcessor
```

Frequencies and Q are positive. Band keys are unique in the non-empty ordered EQ list. Pole count is a
positive even value supported by the canonical validator.

## Dynamics

```veac
compressor_settings(threshold_db: scalar, ratio: scalar, attack_ms: scalar,
                    release_ms: scalar, knee_db: scalar, makeup_gain_db: scalar,
                    mix: percent) -> CompressorSettings
limiter_settings(ceiling_db: scalar, attack_ms: scalar,
                 release_ms: scalar) -> LimiterSettings
gate_settings(threshold_db: scalar, ratio: scalar, attack_ms: scalar,
              release_ms: scalar, range_db: scalar) -> GateSettings
loudness_target(integrated_lufs: scalar, true_peak_dbtp: scalar,
                loudness_range_lu: scalar) -> LoudnessTarget
audio_compressor(key: identifier, settings: CompressorSettings) -> AudioProcessor
audio_limiter(key: identifier, settings: LimiterSettings) -> AudioProcessor
audio_gate(key: identifier, settings: GateSettings) -> AudioProcessor
audio_loudness(key: identifier, target: LoudnessTarget) -> AudioProcessor
```

Ratios are at least one. Attack, release, knee, range, and loudness range are non-negative. Mix is
normalized. These complete descriptors prevent partially configured dynamics processors.

## Crossfade

```veac
audio_fade_linear() -> AudioFadeCurve
audio_fade_equal_power() -> AudioFadeCurve
audio_fade_exponential() -> AudioFadeCurve
audio_crossfade_none() -> AudioCrossfadeChoice
audio_crossfade_present(fade_in: time, fade_out: time,
                        curve: AudioFadeCurve) -> AudioCrossfadeChoice
```

Fade durations are non-negative and cannot exceed the Item record duration. Absence is an explicit
closed choice; it does not mean two zero-duration fades.

## Complete Style

```veac
audio_style(gain: ScalarAnimation, pan: ScalarAnimation, playback: AudioPlayback,
            processors: list<AudioProcessor>, crossfade: AudioCrossfadeChoice) -> AudioStyle
Item.with_audio(style: AudioStyle) -> Item
```

`Item.with_audio` is `GraphEmit`. Processor list order is signal-chain order and processor keys are
unique within the Item. Normalize in `AudioPlayback` denotes the canonical clip policy; the separate
`audio_normalize_effect` is a keyed built-in effect and must not be silently substituted for it.
