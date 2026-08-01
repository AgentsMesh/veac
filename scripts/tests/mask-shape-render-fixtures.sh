#!/usr/bin/env bash

mask_json() {
  local shape=$1 x=$2 y=$3 sx=$4 sy=$5 rotation=$6
  jq -nc --arg shape "$shape" --argjson x "$x" --argjson y "$y" --argjson sx "$sx" \
    --argjson sy "$sy" --argjson rotation "$rotation" '{shape:{type:$shape},
      position:{type:"constant",value:{x:$x,y:$y}},scale:{type:"constant",value:{x:$sx,y:$sy}},
      rotation_degrees:{type:"constant",value:$rotation},feather_pixels:{type:"constant",value:8},invert:false}'
}

write_mask_shape_canonical() {
  local file=$1 heart star linear mirror
  heart=$(mask_json heart .5 .47 .65 .65 0); star=$(mask_json star .5 .48 .65 .65 8)
  linear=$(mask_json linear .5 .5 .6 .6 20); mirror=$(mask_json mirror .5 .5 .6 .6 -12)
  jq -nc --argjson heart "$heart" --argjson star "$star" --argjson linear "$linear" \
    --argjson mirror "$mirror" '{project:{sequences:[{id:"seq_main",tracks:[{clips:[
      {id:"itm_heart",record_range:{start:{timescale:1000,value:0},duration:{timescale:1000,value:1000}},visual:{masks:[$heart]}},
      {id:"itm_star",record_range:{start:{timescale:1000,value:1000},duration:{timescale:1000,value:1000}},visual:{masks:[$star]}},
      {id:"itm_linear",record_range:{start:{timescale:1000,value:2000},duration:{timescale:1000,value:1000}},visual:{masks:[$linear]}},
      {id:"itm_mirror",record_range:{start:{timescale:1000,value:3000},duration:{timescale:1000,value:1000}},visual:{masks:[$mirror]}}
    ]}]}],render_configs:[{id:"out_preview",sequence_id:"seq_main",deliverables:[{id:"dlv_preview",
      target:{type:"file",name:"preview.mp4"},kind:{type:"video",settings:{video:{codec:"h264"}}}}]}]}}' >"$file"
}

make_mask_segment() {
  local file=$1 condition=$2 r=$3 g=$4 b=$5
  ffmpeg -nostdin -v error -y -f lavfi -i 'nullsrc=s=480x270:r=12:d=1' \
    -vf "geq=r='if($condition,$r,16)':g='if($condition,$g,24)':b='if($condition,$b,32)',format=yuv420p" \
    -c:v libx264 -preset ultrafast "$file"
}

make_mask_shape_video() {
  local dir=$1 mode=${2:-valid} tmp
  tmp="$dir/rendered/parts"
  local heart='lte(pow(pow((X-240)/100,2)+pow((135-Y)/100,2)-1,3)-pow((X-240)/100,2)*pow((135-Y)/100,3),0)'
  local star='lte(hypot((cos(.1396)*(X-240)+sin(.1396)*(Y-130))/312,(-sin(.1396)*(X-240)+cos(.1396)*(Y-130))/175.5),0.28+0.12*cos(5*atan2((-sin(.1396)*(X-240)+cos(.1396)*(Y-130))/175.5,(cos(.1396)*(X-240)+sin(.1396)*(Y-130))/312)))'
  local linear='gte(cos(0.349)*(X-240)+sin(0.349)*(Y-135),0)'
  local mirror='lte(abs(cos(-0.209)*(X-240)+sin(-0.209)*(Y-135)),144)'
  case $mode in
    wrong-heart) heart='lt(abs(X-240),100)*lt(abs(Y-135),75)' ;;
    wrong-star) star='lte(hypot((X-240)/312,(Y-130)/175.5),0.32)' ;;
    wrong-linear) linear=1 ;;
    wrong-mirror) mirror="$linear" ;;
  esac
  mkdir -p "$tmp"
  make_mask_segment "$tmp/heart.mp4" "$heart" 255 71 126
  make_mask_segment "$tmp/star.mp4" "$star" 255 209 102
  make_mask_segment "$tmp/linear.mp4" "$linear" 45 212 191
  make_mask_segment "$tmp/mirror.mp4" "$mirror" 122 162 247
  ffmpeg -nostdin -v error -y -i "$tmp/heart.mp4" -i "$tmp/star.mp4" -i "$tmp/linear.mp4" \
    -i "$tmp/mirror.mp4" -filter_complex '[0:v][1:v][2:v][3:v]concat=n=4:v=1:a=0[v]' \
    -map '[v]' -c:v libx264 -preset ultrafast -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  rm -rf "$tmp"
}

make_mask_shape_fixture() {
  local dir="$1/mask-shape-gallery" author preview plan
  mkdir -p "$dir/project" "$dir/plans/preview" "$dir/rendered"
  printf 'mask shape fixture\n' >"$dir/project/main.veac"
  printf 'mask shape preview fixture\n' >"$dir/project/main.preview.veac"
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_preview)
  write_mask_shape_canonical "$author"; cp "$author" "$preview"
  jq '{sequences:.project.sequences,output:{id:"pout_preview",render_config_id:"out_preview",sequence_id:"seq_main"}}' \
    "$preview" >"$plan"
  make_mask_shape_video "$dir"
}
