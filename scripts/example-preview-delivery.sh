#!/usr/bin/env bash

delivery_config_error() {
  echo "examples preview: $*" >&2
  return 1
}

delivery_config_id() {
  local canonical=$1 logical_key=$2 config
  [[ $logical_key =~ ^[a-z][a-z0-9-]*$ ]] ||
    { delivery_config_error "unsafe delivery logical key: $logical_key"; return 1; }
  config=$(jq -er --arg key "$logical_key" '
    .project.authorship.deliveries as $deliveries |
    if ($deliveries | type) != "array" then
      error("missing delivery provenance")
    else
      [$deliveries[] |
        select(.entity.logical_path[-1] == $key) | .render_config_id] as $matches |
      if ($matches | length) == 1 then $matches[0]
      else error("delivery logical key is not unique") end
    end
  ' "$canonical") ||
    { delivery_config_error "cannot resolve delivery logical key: $logical_key"; return 1; }
  jq -e --arg id "$config" '
    [.project.render_configs[] | select(.id == $id)] | length == 1
  ' "$canonical" >/dev/null ||
    { delivery_config_error "delivery provenance points to a missing config: $config"; return 1; }
  printf '%s\n' "$config"
}

canonical_delivery_artifacts() {
  local canonical=$1 config=$2
  jq -ce --arg id "$config" '
    [.project.render_configs[] | select(.id == $id) | .deliverables[] |
      {kind: .kind.type, target_type: .target.type,
       target: (if .target.type == "image_sequence" then .target.pattern
                else .target.name end)}] |
    sort_by(.kind, .target_type, .target)
  ' "$canonical"
}
