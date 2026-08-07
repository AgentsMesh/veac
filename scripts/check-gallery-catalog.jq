def nonempty:
  type == "string" and test("\\S");

def chinese_presentation:
  nonempty and test("[一-龥]");

def without_standard_tokens:
  gsub("VEAC|AgentsMesh|Apple ProRes|ALAC|AAC|BT\\.709|BT\\.2020|GIF|H\\.264|H\\.265|HDR|HEVC|HLS|HSL|LUFS|LUT|MP3|MOV|Main10|Opus|PCM|PNG|PQ|RGB|VP9|WAV|WebM|WebVTT"; "");

def chinese_explanation:
  chinese_presentation and
  ((without_standard_tokens | test("[A-Za-z]{2,}")) | not);

def chinese_cue:
  chinese_presentation and
  (startswith("产物 - ") or chinese_explanation);

def exact_keys($expected):
  (keys_unsorted | sort) == ($expected | sort);

def artifact_key:
  if .kind == "delivery" then "delivery:" + .logical_key else .kind end;

def delivery_artifact:
  exact_keys(["kind", "target", "target_type"]) and
  (.kind as $kind | [
    "adaptive_package", "animated_image", "audio_file", "audio_stem",
    "caption_sidecar", "image_sequence", "scope", "still_image", "video"
  ] | index($kind) != null) and
  (.target_type as $target | ["file", "image_sequence", "package"] |
    index($target) != null) and
  (.target | nonempty and
    test("^[A-Za-z0-9][A-Za-z0-9._%+-]*$") and
    contains("..") | not);

def expected_artifact:
  . as $artifact
  | if $artifact.kind == "delivery" then
      ($artifact | exact_keys(["artifacts", "kind", "logical_key"])) and
      ($artifact.logical_key | nonempty and test("^[a-z][a-z0-9-]*$")) and
      ($artifact.artifacts | type == "array" and length > 0 and
        all(.[]; delivery_artifact) and
        (map([.kind, .target_type, .target] | join(":")) |
          length == (unique | length)))
    else
      ($artifact | exact_keys(["kind"])) and
      ($artifact.kind as $kind | [
        "canonical_project", "resolved_plan", "provider_observations",
        "preview_canonical_project", "preview_resolved_plan", "caption_sidecar",
        "template_bindings", "edit_batch", "edit_outcome", "edit_replay_outcome",
        "probe_snapshot", "source_revision",
        "source_index", "source_edit_batch", "source_edit_outcome"
      ] | index($kind) != null)
    end;

def preview_window:
  exact_keys(["duration_seconds", "start_seconds"]) and
  (.start_seconds | type == "number" and . == 0) and
  (.duration_seconds | type == "number" and . > 0);

def evidence_owner:
  def present: nonempty or (type == "array" and length > 0);
  [.example, (.workflow_evidence // null), (.external_reason // null),
    (.not_applicable // null)] | map(select(present)) | length == 1;

def source_artifacts:
  all(.expected_artifacts[];
    .kind == "canonical_project" or .kind == "preview_canonical_project" or
    .kind == "preview_resolved_plan" or .kind == "delivery" or
    .kind == "source_revision" or .kind == "source_index" or
    .kind == "source_edit_batch" or .kind == "source_edit_outcome" or
    .kind == "edit_batch" or .kind == "edit_outcome" or
    .kind == "edit_replay_outcome" or .kind == "probe_snapshot") and
  ([.expected_artifacts[] | select(.kind == "canonical_project")] | length == 1) and
  ([.expected_artifacts[] | select(.kind == "preview_canonical_project")] | length == 1) and
  ([.expected_artifacts[] | select(.kind == "preview_resolved_plan")] | length == 1) and
  ([.expected_artifacts[] | select(.kind == "delivery")] | length == 1) and
  ([.expected_artifacts[] | select(
    .kind == "source_revision" or .kind == "source_index" or
    .kind == "source_edit_batch" or .kind == "source_edit_outcome")] |
    length == 0 or
    (length == 4 and
      ([.[].kind] | sort) ==
      ["source_edit_batch", "source_edit_outcome", "source_index", "source_revision"])) and
  ([.expected_artifacts[] | select(
    .kind == "edit_batch" or .kind == "edit_outcome" or
    .kind == "edit_replay_outcome")] |
    length == 0 or
    (length == 3 and
      ([.[].kind] | sort) ==
      ["edit_batch", "edit_outcome", "edit_replay_outcome"])) and
  ([.expected_artifacts[] | select(.kind == "probe_snapshot")] | length <= 1);

def presentation_check:
  exact_keys(["cue", "expect"]) and
  (.cue | chinese_cue) and (.expect | chinese_explanation);

def example:
  exact_keys(["checks", "id", "source", "summary", "title"]) and
  (.id | nonempty and test("^[a-z][a-z0-9-]+$")) and
  (.source == ("examples/" + .id + "/main.veac")) and
  (.title | chinese_explanation) and (.summary | chinese_explanation) and
  (.checks | type == "array" and length > 0 and all(.[]; presentation_check));

def target:
  . as $target
  | (all(keys_unsorted[]; . as $key |
      ["id", "kind", "capability_ids", "example", "preview_window",
       "frontend", "expected_artifacts", "workflow_evidence", "external_reason",
       "not_applicable"] | index($key) != null)) and
    ($target.id | nonempty and test("^[a-z][a-z0-9-]+$")) and
    ($target.kind == "render" or $target.kind == "workflow") and
    ($target.capability_ids | type == "array" and length > 0 and
      all(.[]; nonempty and test("^(P1|P2)-[0-9]{2}$"))) and
    ($target.expected_artifacts | type == "array" and length > 0 and
      all(.[]; expected_artifact) and
      (map(artifact_key) | length == (unique | length))) and
    ($target | evidence_owner) and
    ($target.example | nonempty) and
    ($target.example == ("examples/" + $target.id + "/main.veac")) and
    ($target.frontend == "executable") and
    ($target | source_artifacts) and
    (if any($target.expected_artifacts[];
      .kind == "edit_batch" or .kind == "edit_outcome" or
      .kind == "edit_replay_outcome") then
        $target.kind == "workflow" and
        ($target.capability_ids | index("P1-04")) != null and
        ($target.capability_ids | index("P1-08")) != null
      else true end) and
    (if any($target.expected_artifacts[]; .kind == "probe_snapshot") then
        $target.kind == "workflow" and
        ($target.capability_ids | index("P1-34")) != null
      else true end) and
    (if $target.kind == "render" then
      ($target.example | nonempty) and ($target.preview_window | preview_window)
    else $target.preview_window == null end);

def unique_ids:
  ([.[].id] | length) == ([.[].id] | unique | length);

def examples_are_closed:
  ([.examples[].source] | sort)
  == ([.targets[] | select(.example | nonempty) | .example] | sort);

def checks_fit_preview_windows:
  . as $root |
  all($root.targets[] | select(.kind == "render");
    . as $target |
    ($root.examples[] | select(.source == $target.example)) as $example |
    ([ $example.checks[].cue |
      scan("[0-9]+(?:\\.[0-9]+)?") | tonumber ] | max) as $cue_end |
    ($cue_end == null or
      $cue_end <= ($target.preview_window.start_seconds +
        $target.preview_window.duration_seconds)));

exact_keys(["examples", "presentation_language", "schema_version", "targets"]) and
.schema_version == 10 and
.presentation_language == "zh-CN" and
(.examples | type == "array" and length > 0 and unique_ids) and
all(.examples[]; example) and
(.targets | type == "array" and length > 0 and unique_ids) and
all(.targets[]; target) and
examples_are_closed and
checks_fit_preview_windows
