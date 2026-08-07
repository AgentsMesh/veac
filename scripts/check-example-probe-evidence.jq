def one($label):
  if length == 1 then .[0] else error("expected one " + $label) end;

($authoring[0]) as $project |
($plan[0]) as $resolved |
([$project.project.materials[] |
  select(.authorship.logical_path[-1] == "source" and .kind == "video")]
  | one("authored source material")) as $material |
([$resolved.inputs[] | select(.material_id == $material.id)]
  | one("resolved source input")) as $input |
($material.stream_intent.video.global_index) as $video_index |
. as $snapshot |

$snapshot.schema_version == 3 and
($snapshot.engine | type == "string" and length > 0) and
($snapshot.selection_policy | type == "string" and length > 0) and
$snapshot.observed_identity == $material.identity and
($snapshot.streams | type == "array" and length >= 2) and
any($snapshot.streams[];
  .global_index == $video_index and .media_type == "video" and .video != null) and
any($snapshot.streams[]; .media_type == "audio" and .audio != null) and
$material.stream_intent.video.type == "global_index" and
$material.stream_intent.audio.type == "disabled" and
$snapshot.selected_video_stream == $input.video.selection and
$snapshot.selected_video_stream.global_index == $video_index and
$snapshot.selected_audio_stream == null and
$input.canonical_uri == $material.source.uri and
$input.observed_identity == $material.identity and
$input.audio == null and
$input.probe == ($snapshot |
  {schema_version, engine, selection_policy, container_format, container_duration})
