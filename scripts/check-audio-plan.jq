def t($value): {"timescale": 1000, "value": $value};
def r($start; $duration): {"start": t($start), "duration": t($duration)};
def track($id):
  [.sequences[].tracks[] | select(.id == $id)]
  | if length == 1 then .[0] else error("expected one track: \($id)") end;
def clip($id):
  [.sequences[].tracks[].clips[] | select(.id == $id)]
  | if length == 1 then .[0] else error("expected one clip: \($id)") end;

(track("trk_dialogue")) as $dialogue
| (track("trk_music")) as $music_track
| (track("trk_key")) as $key_track
| (clip("itm_voiceover")) as $voice
| (clip("itm_music-bed")) as $music
| (clip("itm_gated-tone")) as $gated
| (clip("itm_normalized-tone")) as $loudness
| (clip("itm_normalized-effect-tone")) as $effect
| any(.output.deliverables[];
    .kind.type == "video" and .kind.settings.audio == {
      "codec": "aac", "sample_rate": 48000, "channels": 2
    })
and $dialogue.routing.audio == {"type":"bus", "bus_id":"bus_dialogue-bus"}
and $music_track.routing.audio == {"type":"bus", "bus_id":"bus_music-bus"}
and $key_track.routing.audio == {"type":"bus", "bus_id":"bus_key-bus"}
and $voice.record_range == r(0; 2000)
and $voice.source_mapping.time_map.source_range_per_repeat.start == t(0)
and $voice.audio.gain.type == "keyframes"
and ($voice.audio.gain.keyframes | map(.time)) == [t(0), t(1000)]
and $voice.audio.pan.value == -0.65
and $voice.audio.normalize and $voice.audio.pitch_policy == "preserve"
and $voice.audio.crossfade.curve == "equal_power"
and ($voice.audio.processors | map(.type)) == ["high_pass", "parametric_eq", "limiter"]
and $music.audio.sidechain.relation_id == "rel_duck"
and $music.audio.sidechain.source == {"type":"bus", "bus_id":"bus_key-bus"}
and ($music.audio.processors | map(.type)) == ["compressor"]
and $gated.record_range == r(2000; 2000)
and $gated.source_mapping.time_map.rate == {"numerator":2, "denominator":1}
and $gated.source_mapping.time_map.source_range_per_repeat.duration == t(4000)
and $gated.audio.pitch_policy == "follow_speed"
and $gated.audio.crossfade.curve == "linear"
and ($gated.audio.processors | map(.type)) == ["gate"]
and $loudness.record_range == r(4000; 2000)
and $loudness.source_mapping.time_map.source_range_per_repeat.start == t(4000)
and $loudness.audio.crossfade.curve == "exponential"
and ($loudness.audio.processors | map(.type)) == ["loudness"]
and $effect.record_range == r(6000; 2000)
and $effect.source_mapping.time_map.source_range_per_repeat.start == t(6000)
and any($effect.effects[];
  .effect_type == "audio.normalize" and
  .parameters.target_lufs == {"type":"number", "value":-18})
