def nonempty:
  type == "string" and length > 0;

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
        "canonical_project",
        "resolved_plan",
        "provider_observations",
        "caption_sidecar",
        "template_bindings",
        "edit_outcome",
        "probe_snapshot"
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

exact_keys(["schema_version", "targets"]) and
.schema_version == 3 and
(.targets | type == "array" and length > 0 and all(.[]; target))
