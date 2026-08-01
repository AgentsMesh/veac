#!/usr/bin/env bash

mask_shape_canonical_contract() {
  local canonical=$1 label=$2 root=${3:-.project}
  jq -e --arg root "$root" '
    def t($v): {"timescale":1000,"value":$v};
    def clips: if $root == ".project" then .project.sequences[].tracks[].clips[]
      else .sequences[].tracks[].clips[] end;
    def clip($id): first(clips | select(.id == $id));
    def constant($v): {"type":"constant","value":$v};
    def contract($id;$start;$shape;$x;$y;$sx;$sy;$rotation):
      clip($id) as $clip | $clip.record_range == {"start":t($start),"duration":t(1000)} and
      ($clip.visual.masks | length) == 1 and $clip.visual.masks[0] as $mask |
      $mask.shape == {"type":$shape} and $mask.position == constant({"x":$x,"y":$y}) and
      $mask.scale == constant({"x":$sx,"y":$sy}) and
      $mask.rotation_degrees == constant($rotation) and
      $mask.feather_pixels == constant(8) and $mask.invert == false;
    contract("itm_heart";0;"heart";0.5;0.47;0.65;0.65;0) and
    contract("itm_star";1000;"star";0.5;0.48;0.65;0.65;8) and
    contract("itm_linear";2000;"linear";0.5;0.5;0.6;0.6;20) and
    contract("itm_mirror";3000;"mirror";0.5;0.5;0.6;0.6;-12)
  ' "$canonical" >/dev/null || fail "mask-shape-gallery $label mask contract failed"
}

mask_shape_metrics() {
  local video=$1 time=$2 kind=$3
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 -vf format=rgb24 -f rawvideo - |
    od -An -v -tu1 | awk -v width=480 -v height=270 -v kind="$kind" '
      function is_hit(r,g,b) {
        if (kind=="heart") return r>175 && g<145 && b>70
        if (kind=="star") return r>175 && g>135 && b<165
        if (kind=="linear") return r<115 && g>135 && b>110
        return r>65 && g>95 && b>155
      }
      function transitions(y, x,last,value,total) {
        last=hit[y*width]+0
        for (x=1;x<width;x++) { value=hit[y*width+x]+0
          if (value!=last) total++; last=value }
        return total+0
      }
      function row_hits(y, x,total) { for (x=0;x<width;x++) total+=hit[y*width+x]+0; return total+0 }
      { for (i=1;i<=NF;i++) { rgb[channel++]=$i
          if (channel==3) { yes=is_hit(rgb[0],rgb[1],rgb[2]); hit[pixel]=yes
            x=pixel%width; y=int(pixel/width)
            if (yes) { count++; if (!seen||x<minx) minx=x; if (!seen||x>maxx) maxx=x
              if (!seen||y<miny) miny=y; if (!seen||y>maxy) maxy=y
              if (x<width/2) left++; else right++; seen=1 }
            channel=0; pixel++ } } }
      END {
        center=hit[135*width+240]+0; left_edge=0; right_edge=0
        for (y=0;y<height;y++) { left_edge+=hit[y*width]+0; right_edge+=hit[y*width+width-1]+0 }
        upper=int(miny+(maxy-miny)*.45); lower=int(miny+(maxy-miny)*.9)
        topology=0; lobe_hits=0
        for (y=miny;y<=upper;y++) { edges=transitions(y); hits=row_hits(y)
          if (edges>topology) topology=edges
          if (edges>=4 && hits>lobe_hits) lobe_hits=hits }
        tl=hit[20*width+20]+0; br=hit[250*width+460]+0; spikes=0; gaps=0
        if (kind=="star") for (i=0;i<5;i++) {
          theta=i*2*3.14159265/5; rot=8*3.14159265/180
          lx=312*.34*cos(theta); ly=175.5*.34*sin(theta)
          sx=int(240+cos(rot)*lx-sin(rot)*ly+.5); sy=int(129.6+sin(rot)*lx+cos(rot)*ly+.5)
          spikes+=hit[sy*width+sx]+0
          theta+=(3.14159265/5); lx=312*.25*cos(theta); ly=175.5*.25*sin(theta)
          sx=int(240+cos(rot)*lx-sin(rot)*ly+.5); sy=int(129.6+sin(rot)*lx+cos(rot)*ly+.5)
          gaps+=hit[sy*width+sx]+0
        }
        print count+0,minx+0,maxx+0,miny+0,maxy+0,center,left_edge,right_edge,
          topology,transitions(135),lobe_hits,row_hits(135),row_hits(lower),
          spikes,gaps,left+0,right+0,tl,br
      }'
}

mask_assert_heart() {
  local values=()
  read -r -a values <<<"$(mask_shape_metrics "$1" 0.5 heart)"
  local count=${values[0]} width=$((values[2]-values[1]+1)) height=$((values[4]-values[3]+1))
  ((count > 18000 && width > 150 && height > 125 && values[8] >= 4 && values[9] == 2 &&
    values[10] > values[12] * 2)) || fail "mask heart rendered topology is wrong: ${values[*]}"
}

mask_assert_star() {
  local values=() area box ratio
  read -r -a values <<<"$(mask_shape_metrics "$1" 1.5 star)"
  area=${values[0]}; box=$(((values[2]-values[1]+1)*(values[4]-values[3]+1)))
  ratio=$((area*100/box))
  ((area > 7000 && values[5] == 1 && ratio >= 30 && ratio <= 68 &&
    values[13] == 5 && values[14] == 0)) || fail "mask star rendered topology is wrong: ${values[*]}"
}

mask_assert_linear() {
  local values=() count
  read -r -a values <<<"$(mask_shape_metrics "$1" 2.5 linear)"; count=${values[0]}
  ((count > 52000 && count < 77000 && values[9] == 1 && values[17] != values[18])) ||
    fail "mask linear rendered half-plane is wrong: ${values[*]}"
}

mask_assert_mirror() {
  local values=() count delta
  read -r -a values <<<"$(mask_shape_metrics "$1" 3.5 mirror)"; count=${values[0]}
  delta=$((values[15]-values[16])); ((delta < 0)) && delta=$((-delta))
  ((count > 62000 && count < 90000 && values[5] == 1 && values[6] == 0 &&
    values[7] == 0 && delta < 1200)) || fail "mask mirror rendered center band is wrong: ${values[*]}"
}

check_mask_shape_gallery_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/mask-shape-gallery"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_preview); video=$(delivery_video_path "$dir" preview preview)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  mask_shape_canonical_contract "$author" authoring
  mask_shape_canonical_contract "$preview" preview
  mask_shape_canonical_contract "$plan" plan .plan
  video_contract "$video" 3.9
  assert_stream_field "$video" v:0 width 480 "mask-shape-gallery video"
  assert_stream_field "$video" v:0 height 270 "mask-shape-gallery video"
  assert_unique_frames "$video" 4 0.5 1.5 2.5 3.5
  mask_assert_heart "$video"; mask_assert_star "$video"; mask_assert_linear "$video"; mask_assert_mirror "$video"
}
