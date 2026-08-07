#!/usr/bin/env bash

executable_local_image_project_contract() {
  local file=$1 poster=$2 material=$3
  jq -e --arg poster "$poster" --arg material "$material" '
    def clip($id): first(.project.sequences[].tracks[].clips[]|select(.id==$id));
    first(.project.materials[]|select(.id==$material)) as $asset | clip($poster) as $clip |
    $asset.kind=="image" and $asset.source=={type:"file",uri:"assets/executable-local-image.png"} and
    $asset.identity=={algorithm:"sha256",digest:"e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5"} and
    $clip.source=={type:"media",material_id:$material} and
    $clip.visual.frame=={width:{value:56,unit:"pixels"},height:{value:30,unit:"pixels"},fit:"contain"} and
    $clip.visual.placement=={type:"anchor",anchor:"center",inset:{x:0,y:0}} and
    $clip.visual.transform.anchor=={x:.25,y:.5} and
    $clip.visual.transform.flip_horizontal==true and
    $clip.visual.transform.flip_vertical==false and
    $clip.visual.transform.position.value.y=={value:-1,unit:"pixels"} and
    $clip.visual.transform.scale.value=={x:.72,y:.72} and
    $clip.visual.transform.rotation_degrees.value==8
  ' "$file" >/dev/null || fail "executable-local-image project contract failed: $file"
}

check_executable_local_image_evidence() {
  local dir="$PREVIEW_ROOT/executable-local-image"
  [[ -d $dir ]] || return 0
  local canonical preview plan video poster material image
  IFS=$'\t' read -r canonical preview plan video < <(executable_preview_artifacts "$dir")
  poster=$(executable_clip_id "$canonical" image poster)
  material=$(executable_material_id "$canonical" poster)
  image="$dir/project/assets/executable-local-image.png"
  require_file "$image"
  [[ $(shasum -a 256 "$image" | awk '{print $1}') == \
    e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5 ]] ||
    fail "executable-local-image source digest mismatch"
  executable_local_image_project_contract "$canonical" "$poster" "$material"
  executable_local_image_project_contract "$preview" "$poster" "$material"
  jq -e --arg poster "$poster" --arg material "$material" '
    first(.inputs[]|select(.material_id==$material)) as $input |
    first(.sequences[].tracks[].clips[]|select(.id==$poster)) as $clip |
    $input.kind=={type:"media",material_kind:"image"} and
    $input.canonical_uri=="assets/executable-local-image.png" and
    $input.observed_identity.digest=="e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5" and
    $input.video.info.width==64 and $input.video.info.height==36 and
    $input.video.info.pixel_format=="monob" and
    $clip.source=={type:"media",input_id:$input.id,video_stream:{global_index:0,type_index:0},audio_stream:null} and
    $clip.visual.frame.fit=="contain" and $clip.visual.transform.flip_horizontal==true and
    (.sequences[0].duration.value/600)==3 and
    .output.raster=={width:64,height:36,frame_rate:{numerator:12,denominator:1},captions:"discard"}
  ' "$plan" >/dev/null || fail "executable-local-image plan contract failed"
  assert_executable_video "$video" 3 64 36 0 executable-local-image
  assert_non_uniform_image "$image" "executable-local-image source"
  local first_diff second_diff
  first_diff=$(frame_difference_avg "$video" 0.5 1.5)
  second_diff=$(frame_difference_avg "$video" 0.5 2.5)
  awk -v a="$first_diff" -v b="$second_diff" 'BEGIN { exit !(a<.1 && b<.1) }' ||
    fail "local image unexpectedly changes over its three-second hold"
  local left right
  left=$(region_yavg "$video" 1.5 '4:4:28:6')
  right=$(region_yavg "$video" 1.5 '4:4:44:6')
  assert_executable_gt "$right" "$left" 70 "local image horizontal flip is visible"
  local count width height
  read -r count width height < <(frame_bright_bbox "$video" 1.5 64 680)
  ((count>=70 && width>=16 && height>=6)) ||
    fail "local image content is not visible at native 64x36: $count/${width}x$height"
}
