#!/usr/bin/env bash

matte_mask() {
  local shape=$1
  jq -nc --arg shape "$shape" '{shape:{type:$shape},
    position:{type:"constant",value:{x:.5,y:.5}},scale:{type:"constant",value:{x:.6,y:.6}},
    rotation_degrees:{type:"constant",value:0},feather_pixels:{type:"constant",value:8},
    expansion_pixels:{type:"constant",value:0},invert:false}'
}

matte_visual() {
  local masks=$1
  jq -nc --argjson masks "$masks" '{placement:{type:"anchor",anchor:"center",inset:{x:0,y:0}},
    frame:null,transform:{anchor:{x:.5,y:.5}},masks:$masks}'
}

write_masks_canonical() {
  local file=$1 clips='[]' key shape start mask visual
  local keys=(circle rectangle ellipse rounded polygon path)
  local shapes=(circle rectangle ellipse rounded_rectangle polygon path)
  for index in {0..5}; do
    key=${keys[index]}; shape=${shapes[index]}; start=$index
    mask=$(matte_mask "$shape"); visual=$(matte_visual "[$mask]")
    clips=$(jq -nc --argjson clips "$clips" --arg key "$key" --arg shape "$shape" \
      --argjson start "$start" --argjson visual "$visual" '$clips+[{
        id:("itm_"+$key),authorship:{logical_path:["masks-and-mattes","main","visual",$key],events:[]},
        record_range:{start:{timescale:1,value:$start},duration:{timescale:1,value:1}},visual:$visual}]')
  done
  local masks
  masks=$(jq -nc '["circle","rectangle","ellipse","rounded_rectangle","polygon","path"] |
    map({shape:{type:.},position:{type:"constant",value:{x:.5,y:.5}}})')
  visual=$(matte_visual "$masks")
  clips=$(jq -nc --argjson clips "$clips" --argjson visual "$visual" '$clips+[{
    id:"itm_combined",authorship:{logical_path:["masks-and-mattes","main","visual","combined"],events:[]},
    record_range:{start:{timescale:1,value:6},duration:{timescale:1,value:2}},visual:$visual}]')
  jq -nc --argjson clips "$clips" '{project:{authorship:{entity:{logical_path:["masks-and-mattes"],events:[]},
    multicam_groups:[],annotations:[],deliveries:[{render_config_id:"out_preview",
    entity:{logical_path:["masks-and-mattes","preview"],events:[]}}]},sequences:[{
    id:"seq_main",authorship:{type:"veac",entity:{logical_path:["masks-and-mattes","main"],events:[]},
    tracks:[],relations:[],applies:[]},tracks:[{clips:$clips}]}],render_configs:[{
    id:"out_preview",sequence_id:"seq_main",deliverables:[{id:"dlv_preview",
    target:{type:"file",name:"preview.mp4"},kind:{type:"video",settings:{video:{codec:"h264"}}}}]}]}}' >"$file"
}

make_masks_segment() {
  local file=$1 condition=$2 color=$3
  ffmpeg -nostdin -v error -y -f lavfi -i 'nullsrc=s=480x270:r=12:d=1' \
    -vf "geq=r='if($condition,0x${color:0:2},16)':g='if($condition,0x${color:2:2},24)':b='if($condition,0x${color:4:2},32)',format=yuv420p" \
    -c:v libx264 -preset ultrafast "$file"
}

make_masks_video() {
  local dir=$1 mode=${2:-valid} tmp condition
  tmp="$dir/rendered/parts"
  local conditions=('lte(hypot(X-240,Y-135),78)' 'lt(abs(X-240),105)*lt(abs(Y-135),68)'
    'lte(pow((X-240)/105,2)+pow((Y-135)/60,2),1)'
    'lt(abs(X-240),100)*lt(abs(Y-135),65)'
    'lte(abs(X-240)/115+abs(Y-135)/82,1)'
    'gte(Y,65)*lte(Y,220)*gte(X,140+(Y-65)*.3)*lte(X,340-(Y-65)*.3)'
    'lte(hypot(X-240,Y-135),92)' 'lte(hypot(X-240,Y-135),70)'
    'gte(hypot(X-240,Y-135),72)' 'gte(hypot(X-240,Y-135),96)')
  local colors=(ff2d95 00b4d8 ffc857 7bd389 9b5de5 f15bb5 ff006e ff006e 00bbf9 00bbf9)
  mkdir -p "$tmp"
  for index in {0..9}; do
    condition=${conditions[index]}
    [[ $mode != off-center || $index -gt 7 ]] || condition=${condition//240/190}
    if [[ $mode == label-only && $index -le 7 ]]; then
      condition='gte(X,220)*lte(X,260)*gte(Y,120)*lte(Y,150)'
    fi
    make_masks_segment "$tmp/$index.mp4" "$condition" "${colors[index]}"
  done
  local inputs='' filter=''
  for index in {0..9}; do inputs+=" -i $tmp/$index.mp4"; filter+="[$index:v]"; done
  # shellcheck disable=SC2086
  ffmpeg -nostdin -v error -y $inputs -filter_complex "${filter}concat=n=10:v=1:a=0[v]" \
    -map '[v]' -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  rm -rf "$tmp"
}

make_masks_fixture() {
  local dir="$1/masks-and-mattes" author preview plan
  mkdir -p "$dir/project" "$dir/plans/preview" "$dir/rendered"
  printf 'masks fixture\n' >"$dir/project/main.veac"
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_preview)
  write_masks_canonical "$author"; cp "$author" "$preview"
  jq '{sequences:.project.sequences,output:{id:"pout_preview",render_config_id:"out_preview",
    sequence_id:"seq_main"}}' "$preview" >"$plan"
  make_masks_video "$dir"
}
