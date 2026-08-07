#!/usr/bin/env bash

executable_centered_dissolve_project_contract() {
  local file=$1 outgoing=$2 incoming=$3 relation=$4 mix_label=$5
  jq -e --arg outgoing "$outgoing" --arg incoming "$incoming" \
    --arg relation "$relation" --arg mix "$mix_label" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[]|select(.id==$id));
    first(.project.relations[]|select(.id==$relation)).kind as $transition |
    $transition.type=="transition" and
    $transition.from=={type:"item",item_id:$outgoing} and
    $transition.to=={type:"item",item_id:$incoming} and
    $transition.transition=={kind:{type:"dissolve"},duration:{timescale:600,value:240},alignment:"centered"} and
    [(clip($outgoing).record_range.start|seconds),(clip($outgoing).record_range.duration|seconds),
      (clip($incoming).record_range.start|seconds),(clip($incoming).record_range.duration|seconds)]==[0,2,1.6,2.4] and
    clip($mix).source.text=="居中叠化：红色与蓝色同时存在"
  ' "$file" >/dev/null || fail "executable-centered-dissolve project contract failed: $file"
}

check_executable_centered_dissolve_evidence() {
  local dir="$PREVIEW_ROOT/executable-centered-dissolve"
  [[ -d $dir ]] || return 0
  local canonical preview plan video outgoing incoming relation mix_label
  IFS=$'\t' read -r canonical preview plan video < <(executable_preview_artifacts "$dir")
  assert_resolution_chain "$dir" "$plan" "$video" executable-centered-dissolve
  outgoing=$(executable_clip_id "$canonical" scenes scene-a)
  incoming=$(executable_clip_id "$canonical" scenes scene-b)
  mix_label=$(executable_clip_id "$canonical" labels label-mix)
  relation=$(executable_relation_id "$canonical" scene-cut)
  executable_centered_dissolve_project_contract "$canonical" "$outgoing" "$incoming" \
    "$relation" "$mix_label"
  executable_centered_dissolve_project_contract "$preview" "$outgoing" "$incoming" \
    "$relation" "$mix_label"
  jq -e --arg outgoing "$outgoing" --arg incoming "$incoming" --arg relation "$relation" '
    first(.sequences[].tracks[].transitions[]|select(.relation_id==$relation)) == {
      relation_id:$relation,outgoing_clip_id:$outgoing,incoming_clip_id:$incoming,
      kind:{type:"dissolve"},alignment:"centered",
      record_window:{start:{timescale:600,value:960},duration:{timescale:600,value:240}},
      cut_time:{timescale:600,value:1080},
      outgoing_range:{start:{timescale:600,value:960},duration:{timescale:600,value:240}},
      incoming_range:{start:{timescale:600,value:0},duration:{timescale:600,value:240}}
    } and (.sequences[0].duration.value/600)==4 and
    all([first(.sequences[].tracks[].clips[]|select(.id==$outgoing)),
      first(.sequences[].tracks[].clips[]|select(.id==$incoming))][];
      .source_mapping==null)
  ' "$plan" >/dev/null || fail "executable-centered-dissolve plan contract failed"
  assert_executable_video "$video" 4 480 270 0 executable-centered-dissolve
  assert_executable_rgb "$video" 0.5 '40:30:10:10' '239 68 68' 18 "dissolve red endpoint"
  assert_executable_rgb "$video" 2.5 '40:30:10:10' '37 99 235' 18 "dissolve blue endpoint"
  local red_a red_b red_c blue_a blue_b blue_c
  read -r red_a _ blue_a <<<"$(region_rgb "$video" 1.7 '40:30:10:10')"
  read -r red_b _ blue_b <<<"$(region_rgb "$video" 1.8 '40:30:10:10')"
  read -r red_c _ blue_c <<<"$(region_rgb "$video" 1.9 '40:30:10:10')"
  awk -v ra="$red_a" -v rb="$red_b" -v rc="$red_c" \
    -v ba="$blue_a" -v bb="$blue_b" -v bc="$blue_c" '
    BEGIN { exit !(ra>rb && rb>rc && ba<bb && bb<bc) }
  ' || fail "centered dissolve is not monotonic across the full overlap"
  assert_executable_rgb "$video" 1.8 '40:30:10:10' '138 84 152' 28 "dissolve midpoint mix"
}
