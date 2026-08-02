def resized($edge):
  . as $source
  | if $source.width >= $source.height then
      .width = $edge
      | .height = ([2, (((($source.height * $edge) / $source.width) / 2 | floor) * 2)] | max)
    else
      .height = $edge
      | .width = ([2, (((($source.width * $edge) / $source.height) / 2 | floor) * 2)] | max)
    end;

def preview_rate($fps):
  .frame_rate = {numerator: $fps, denominator: 1};

def seconds($context):
  if type == "object" and (.value | type) == "number" and
      (.timescale | type) == "number" and .timescale > 0 then
    .value / .timescale
  else error("invalid canonical time in \($context): \(.)") end;

def clamp_keyframes($duration):
  walk(
    if type == "object" and .type? == "keyframes" then
      .keyframes |= map(select((.time | seconds("keyframe")) <= $duration))
    else . end
  );

def trim_source_segments($remaining):
  if length == 0 or $remaining <= 0 then []
  else .[0] as $segment
  | ($segment.record_duration.value) as $segment_ticks
  | if $segment_ticks <= $remaining then
      [$segment] + (.[1:] | trim_source_segments($remaining - $segment_ticks))
    else [($segment
      | .record_duration.value = $remaining
      | if .interpolation == "linear" then
          (.source_end.value - .source_start.value) as $source_ticks
          | .source_end.value = (.source_start.value +
              (($source_ticks * $remaining / $segment_ticks) | round))
        else . end)]
    end
  end;

def trim_source_mapping($duration):
  if (.source_mapping? | type) == "object" and
      .source_mapping.time_map.type == "curve" then
    .source_mapping.time_map.segments |= trim_source_segments($duration.value)
  else . end;

def clamp_range($limit):
  . as $range
  | ($range.start | seconds("range start")) as $start
  | (($limit - $start) * $range.duration.timescale | floor) as $remaining
  | .duration.value = ([.duration.value, $remaining] | min);

def clamp_multicam($duration):
  if .source.type == "multicam" then
    .source.switches |= map(
      (.range.start | seconds("multicam switch")) as $start
      | select($start < $duration)
      | (.range |= clamp_range($duration))
    )
  else . end;

def trim_record($limit):
  if (.record_range | type) != "object" then
    error("missing record range for preview item: \(.id // "unknown")")
  else . end
  | (.record_range.start | seconds("record start")) as $start
  | select($start < $limit)
  | (.record_range |= clamp_range($limit))
  | trim_source_mapping(.record_range.duration)
  | (.record_range.duration | seconds("record duration")) as $duration
  | clamp_keyframes($duration)
  | clamp_multicam($duration);

def trim_apply($limit; $items):
  trim_record($limit)
  | (.record_range.duration | seconds("apply duration")) as $duration
  | if .active_range == null then . else
      (.active_range.start | seconds("apply active start")) as $start
      | select($start < $duration)
      | (.active_range |= clamp_range($duration))
    end
  | if .target.type == "item_set" then
      (.target.item_ids |= map(. as $id | select(($items | index($id)) != null)))
      | select((.target.item_ids | length) > 0)
    else . end;

def relation_valid($items; $applies):
  [.. | objects | select(.type? == "item" or .type? == "apply")]
  | all(.[]; . as $endpoint |
      if $endpoint.type == "item" then
        ($items | index($endpoint.item_id)) != null
      else ($applies | index($endpoint.apply_id)) != null end);

def annotation_valid($items):
  if .target.type == "clip" then
    (.target.clip_id as $id | ($items | index($id)) != null)
  else true end;

def apply_window($duration):
  (.project.sequences[].tracks[].clips |= map(trim_record($duration)))
  | ([.project.sequences[].tracks[].clips[].id]) as $items
  | (.project.sequences[].applies |= map(trim_apply($duration; $items)))
  | ([.project.sequences[].applies[].id]) as $applies
  | (.project.relations |= map(select(relation_valid($items; $applies))))
  | (.project.annotations |= map(select(annotation_valid($items))));

def preview_font_material($id; $uri):
  {
    id: $id,
    kind: "font",
    source: {type: "file", uri: $uri},
    identity: null,
    stream_intent: {
      video: {type: "disabled"},
      audio: {type: "disabled"}
    },
    probe: null,
    metadata: {}
  };

def family_font_paths:
  paths(objects |
    .type? == "family" and
    (.family? | type) == "string" and
    (keys_unsorted | sort) == ["family", "type"]);

def preview_font_id($path):
  if ($path | any(. == "fallback_fonts")) then
    "med_preview-arabic-font"
  else "med_preview-font" end;

def preview_font_uri($id):
  if $id == "med_preview-arabic-font" then
    "assets/preview-arabic-font.ttf"
  else "assets/preview-font.ttf" end;

def adapt_family_fonts:
  [family_font_paths] as $paths
  | reduce $paths[] as $path (.;
      setpath($path; {
        type: "material",
        material_id: preview_font_id($path)
      }))
  | ([$paths[] | preview_font_id(.)] | unique) as $material_ids
  | reduce $material_ids[] as $id (.;
      if any(.project.materials[]?; .id == $id) then
        error("reserved preview font material already exists: \($id)")
      else
        .project.materials += [preview_font_material($id; preview_font_uri($id))]
      end);

def valid_inputs:
  ($edge | type == "number" and . > 0 and floor == .) and
  ($fps | type == "number" and . > 0 and floor == .) and
  ($window == null or
    (($window | type) == "object" and
      ($window | keys_unsorted | sort) == ["duration_seconds", "start_seconds"] and
      ($window.start_seconds | type == "number" and . == 0) and
      ($window.duration_seconds | type == "number" and . > 0)));

if valid_inputs | not then error("invalid preview parameters") else . end
| adapt_family_fonts
| .project.sequences[].settings |= preview_rate($fps)
| .project.sequences[].tracks[] |= if .kind == "video" then
    .clips |= map(
      .audio = null
      | (.source_start.seconds? // null) as $start
      | if ($start != null and (($start.numerator / $start.denominator) > 60))
        then .source_start.seconds = {numerator: 0, denominator: 1}
        else . end
    )
  else . end
| .project.render_configs[].raster |= (resized($edge) | preview_rate($fps))
| if $window == null then . else apply_window($window.duration_seconds) end
| (.project.render_configs[].deliverables[]?
    | select(.kind.type == "video")
    | .kind.settings.hardware) = {type: "software"}
