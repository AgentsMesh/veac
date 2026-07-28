def nonempty:
  type == "string" and test("\\S");

def chinese_presentation:
  nonempty and test("[一-龥]");

def without_standard_tokens:
  gsub("VEAC|AgentsMesh|Apple ProRes|ALAC|AAC|BT\\.709|GIF|H\\.264|HSL|LUFS|LUT|PCM|PNG|RGB|WAV|WebVTT"; "");

def chinese_explanation:
  chinese_presentation and
  ((without_standard_tokens | test("[A-Za-z]{2,}")) | not);

def chinese_cue:
  chinese_presentation and
  (startswith("产物 - ") or chinese_explanation);

def exact_keys($expected):
  (keys_unsorted | sort) == ($expected | sort);

def artifact_key:
  if .kind == "authoring_output" then "authoring_output:" + .id else .kind end;

def expected_artifact:
  . as $artifact
  | if $artifact.kind == "authoring_output" then
      ($artifact | exact_keys(["id", "kind"])) and
      ($artifact.id | nonempty and test("^[a-z][a-z0-9-]*$"))
    else
      ($artifact | exact_keys(["kind"])) and
      ($artifact.kind as $kind | [
        "canonical_project", "resolved_plan", "provider_observations",
        "caption_sidecar", "template_bindings", "edit_outcome", "probe_snapshot"
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
    .kind == "canonical_project" or .kind == "resolved_plan" or
    .kind == "authoring_output") and
  ([.expected_artifacts[] | select(.kind == "canonical_project")] | length == 1) and
  ([.expected_artifacts[] | select(.kind == "resolved_plan")] | length == 1) and
  any(.expected_artifacts[]; .kind == "authoring_output");

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
       "expected_artifacts", "workflow_evidence", "external_reason",
       "not_applicable"] | index($key) != null)) and
    ($target.id | nonempty and test("^[a-z][a-z0-9-]+$")) and
    ($target.kind == "render" or $target.kind == "workflow") and
    ($target.capability_ids | type == "array" and length > 0 and
      all(.[]; nonempty and test("^(P1|P2)-[0-9]{2}$"))) and
    ($target.expected_artifacts | type == "array" and length > 0 and
      all(.[]; expected_artifact) and
      (map(artifact_key) | length == (unique | length))) and
    ($target | evidence_owner) and
    (if ($target.example | nonempty) then
      ($target.example == ("examples/" + $target.id + "/main.veac")) and
      ($target | source_artifacts)
    else $target.kind == "workflow" end) and
    (if $target.kind == "render" then
      ($target.example | nonempty) and ($target.preview_window | preview_window)
    else $target.preview_window == null end);

def unique_ids:
  ([.[].id] | length) == ([.[].id] | unique | length);

def examples_are_closed:
  ([.examples[].source] | sort)
  == ([.targets[] | select(.example | nonempty) | .example] | sort);

exact_keys(["examples", "presentation_language", "schema_version", "targets"]) and
.schema_version == 5 and
.presentation_language == "zh-CN" and
(.examples | type == "array" and length > 0 and unique_ids) and
all(.examples[]; example) and
(.targets | type == "array" and length > 0 and unique_ids) and
all(.targets[]; target) and
examples_are_closed
