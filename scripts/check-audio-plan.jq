def only($label):
  if length == 1 then .[0] else error("expected one \($label), got \(length)") end;
def t($value): {"timescale":600,"value":$value};
def r($start; $duration): {"start":t($start),"duration":t($duration)};
def mapping:
  {"frame_synthesis":"nearest","out_of_range":"strict","time_map":{
    "direction":"forward","rate":{"denominator":1,"numerator":1},"repeat":1,
    "source_range_per_repeat":r(0;1200),"type":"linear"}};
def track_contract($track; $order; $routing):
  $track.kind == "audio" and $track.order == $order and $track.source_order == $order and
  $track.placement_mode == "free" and $track.routing == $routing and
  $track.state == {"audio_enabled":true,"include_in_render":true,"visual_enabled":false} and
  $track.transitions == [] and ($track.clips | length) == 1;

$canonical[0] as $source |
def canonical_clip($key):
  [$source.project.sequences[].tracks[].clips[] |
    select(.authorship.logical_path[-1] == $key)] | only("canonical clip " + $key);
def canonical_track($clip):
  [$source.project.sequences[].tracks[] | select(any(.clips[]; .id == $clip))] |
    only("canonical owning track");

canonical_clip("voice") as $canonical_voice |
canonical_clip("music") as $canonical_music |
canonical_clip("key") as $canonical_key |
canonical_track($canonical_voice.id) as $canonical_voice_track |
canonical_track($canonical_music.id) as $canonical_music_track |
canonical_track($canonical_key.id) as $canonical_key_track |
([$source.project.relations[] | select(
  .kind.type == "sidechain" and
  .kind.target == {"item_id":$canonical_music.id,"type":"item"} and
  .kind.key == {"track_id":$canonical_key_track.id,"type":"track"}
)] | only("canonical sidechain relation")) as $canonical_relation |

. as $plan |
[$plan.sequences[] | select(.id == $plan.entry_sequence_id)] | only("entry sequence") as $sequence |
[$sequence.tracks[] | select(any(.clips[]; .id == $canonical_voice.id))] |
  only("voice track") as $voice_track |
[$sequence.tracks[] | select(any(.clips[]; .id == $canonical_music.id))] |
  only("music track") as $music_track |
[$sequence.tracks[] | select(any(.clips[]; .id == $canonical_key.id))] |
  only("key track") as $key_track |
[$voice_track.clips[] | select(.id == $canonical_voice.id)] | only("voice clip") as $voice |
[$music_track.clips[] | select(.id == $canonical_music.id)] | only("music clip") as $music |
[$key_track.clips[] | select(.id == $canonical_key.id)] | only("key clip") as $key |

$plan.header.schema == "https://veac.dev/schemas/render-plan" and
$plan.header.schema_version == 6 and $plan.header.source.timebase == 600 and
$plan.header.resolver.effect_registry_version == "veac-ir-effects-v2" and
$plan.entry_sequence_id == $source.project.entry_sequence_id and
$plan.output.sequence_id == $source.project.entry_sequence_id and
($plan.output.deliverables | length) == 1 and
$plan.output.deliverables[0].kind.type == "video" and
$plan.output.deliverables[0].kind.settings.audio ==
  {"channels":2,"codec":"aac","sample_rate":48000} and
$sequence.duration == t(2400) and
([$sequence.tracks[] | select(.kind == "audio") | .id] ==
  [$canonical_voice_track.id,$canonical_music_track.id,$canonical_key_track.id]) and
track_contract($voice_track;0;{"audio":{"type":"main_mix"},"visual":null}) and
track_contract($music_track;1;{"audio":{"bus_id":$canonical_music_track.routing.bus_id,
  "type":"bus"},"visual":null}) and
track_contract($key_track;2;{"audio":{"type":"main_mix"},"visual":null}) and

$voice.record_range == r(0;1200) and $music.record_range == r(1200;1200) and
$key.record_range == r(1200;1200) and
$voice.source_mapping == mapping and $music.source_mapping == mapping and
$key.source_mapping == mapping and $voice.source == $music.source and
$music.source == $key.source and $voice.source.type == "media" and
$voice.source.video_stream == null and
$voice.source.audio_stream == {"global_index":0,"type_index":0} and

$voice.audio.gain == {"keyframes":[
  {"id":$canonical_voice.audio.gain.keyframes[0].id,"interpolation":{"type":"linear"},
    "time":t(0),"value":0.35},
  {"id":$canonical_voice.audio.gain.keyframes[1].id,"interpolation":{"type":"ease_out"},
    "time":t(600),"value":0.9}],"type":"keyframes"} and
$voice.audio.pan == {"keyframes":[
  {"id":$canonical_voice.audio.pan.keyframes[0].id,"interpolation":{"type":"linear"},
    "time":t(0),"value":-0.4},
  {"id":$canonical_voice.audio.pan.keyframes[1].id,"interpolation":{"type":"ease_in_out"},
    "time":t(600),"value":0.4}],"type":"keyframes"} and
$voice.audio.muted == false and $voice.audio.normalize == false and
$voice.audio.pitch_policy == "preserve" and $voice.audio.sidechain == null and
$voice.audio.crossfade == {"curve":"equal_power","fade_in":t(120),"fade_out":t(120)} and
$voice.effects == [] and
$voice.audio.processors == [
  {"id":$canonical_voice.audio.processors[0].id,"kind":{"bands":[{
    "frequency_hz":3000,"gain_db":2,"id":$canonical_voice.audio.processors[0].kind.bands[0].id,
    "q":1.2}],"type":"parametric_eq"}},
  {"id":$canonical_voice.audio.processors[1].id,
    "kind":{"frequency_hz":80,"poles":2,"q":0.7,"type":"high_pass"}},
  {"id":$canonical_voice.audio.processors[2].id,
    "kind":{"frequency_hz":18000,"poles":2,"q":0.7,"type":"low_pass"}},
  {"id":$canonical_voice.audio.processors[3].id,"kind":{"attack_ms":10,"knee_db":4,
    "makeup_gain_db":2,"mix":0.75,"ratio":3,"release_ms":120,"threshold_db":-18,
    "type":"compressor"}},
  {"id":$canonical_voice.audio.processors[4].id,
    "kind":{"attack_ms":5,"ceiling_db":-1,"release_ms":100,"type":"limiter"}},
  {"id":$canonical_voice.audio.processors[5].id,"kind":{"attack_ms":5,"range_db":-30,
    "ratio":2,"release_ms":80,"threshold_db":-50,"type":"gate"}},
  {"id":$canonical_voice.audio.processors[6].id,"kind":{"integrated_lufs":-16,
    "loudness_range_lu":8,"true_peak_dbtp":-1,"type":"loudness"}}
] and

$music.audio == {"crossfade":{"curve":"linear","fade_in":t(150),"fade_out":t(150)},
  "gain":{"type":"constant","value":0.65},"muted":false,"normalize":false,
  "pan":{"type":"constant","value":-0.2},"pitch_policy":"follow_speed","processors":[],
  "sidechain":{"active_range":null,"attack_ms":10,"ratio":4,
    "relation_id":$canonical_relation.id,"release_ms":120,
    "source":{"track_id":$canonical_key_track.id,"type":"track"},"threshold_db":-24}} and
$music.effects == [{"active_range":r(0;1200),"effect":{
  "target_lufs":-14,"type":"audio_normalize"},"id":$canonical_music.effects[0].id}] and

$key.audio == {"crossfade":{"curve":"exponential","fade_in":t(90),"fade_out":t(90)},
  "gain":{"type":"constant","value":0.5},"muted":false,"normalize":false,
  "pan":{"type":"constant","value":0.2},"pitch_policy":"preserve","processors":[],
  "sidechain":null} and $key.effects == []
