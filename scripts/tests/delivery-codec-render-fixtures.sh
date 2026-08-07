#!/usr/bin/env bash

write_codec_canonical() {
  local file=$1 width=$2 height=$3 fps=$4
  jq -nc --argjson width "$width" --argjson height "$height" --argjson fps "$fps" '{project:{
    authorship:{entity:{logical_path:["delivery-codec-matrix"],events:[]},multicam_groups:[],
      annotations:[],deliveries:[{render_config_id:"out_codec-matrix",
      entity:{logical_path:["delivery-codec-matrix","codec-matrix"],events:[]}}]},
    sequences:[{id:"seq_main",authorship:{type:"veac",entity:{
      logical_path:["delivery-codec-matrix","main"],events:[]},tracks:[],relations:[],applies:[]},
      tracks:[]}],render_configs:[{id:"out_codec-matrix",sequence_id:"seq_main",
    raster:{width:$width,height:$height,frame_rate:{numerator:$fps,denominator:1},captions:"burn_in"},deliverables:[
    {id:"dlv_hevc-main10",target:{type:"file",name:"hevc.mov"},kind:{type:"video",settings:{container:"mov",
      video:{codec:"h265",pixel_format:"yuv420p10le",alpha:"opaque",color_space:{primaries:"bt2020",
      transfer:"smpte2084",matrix:"bt2020_ncl",range:"limited"},rate_control:{type:"crf",value:28},
      gop_size:null,b_frames:null,profile:"h265_main10",level:null},audio:null,optimize_for_streaming:false,
      pass_mode:"single",hardware:{type:"software"}}}},
    {id:"dlv_vp9-opus",target:{type:"file",name:"vp9.webm"},kind:{type:"video",settings:{container:"webm",
      video:{codec:"vp9",pixel_format:"yuv420p",alpha:"opaque",color_space:null,rate_control:{type:"crf",value:30},
      gop_size:null,b_frames:null,profile:"vp9_profile0",level:null},audio:{codec:"opus",sample_rate:48000,channels:2},
      optimize_for_streaming:false,pass_mode:"single",hardware:{type:"software"}}}}]}]}}' >"$file"
}

make_hevc_artifact() {
  local file=$1 mode=${2:-valid} size=480x270 duration=1 source params
  local pix=yuv420p10le profile=main10 primaries=bt2020 transfer=smpte2084 matrix=bt2020nc
  source="nullsrc=s=$size:r=12:d=$duration,format=yuv420p10le,geq=lum='64+876*X/W':cb=512:cr=512"
  case $mode in
    eight-bit) pix=yuv420p; profile=main; source="nullsrc=s=$size:r=12:d=1,format=yuv420p,geq=lum='16+219*X/W':cb=128:cr=128" ;;
    wrong-color) primaries=bt709; transfer=bt709; matrix=bt709 ;;
    low-levels) source="nullsrc=s=$size:r=12:d=1,format=yuv420p10le,geq=lum='64+58*floor(16*X/W)':cb=512:cr=512" ;;
    wrong-size) size=320x180; source="nullsrc=s=$size:r=12:d=1,format=yuv420p10le,geq=lum='64+876*X/W':cb=512:cr=512" ;;
    wrong-duration) duration=0.5; source="nullsrc=s=$size:r=12:d=$duration,format=yuv420p10le,geq=lum='64+876*X/W':cb=512:cr=512" ;;
  esac
  params="lossless=1:pools=1:frame-threads=1:log-level=error:colorprim=$primaries:transfer=$transfer:colormatrix=$matrix:range=limited"
  ffmpeg -nostdin -v error -y -f lavfi -i "$source" -an -c:v libx265 -pix_fmt "$pix" \
    -profile:v "$profile" -x265-params "$params" \
    -color_primaries "$primaries" -color_trc "$transfer" -colorspace "$matrix" -color_range tv \
    -tag:v hvc1 -movflags +faststart -f mov "$file"
}

make_vp9_artifact() {
  local file=$1 mode=${2:-valid} size=480x270 duration=1 channels=2 audio='sine=frequency=523:sample_rate=48000:duration=1'
  local codec=libvpx-vp9 format=webm profile=0
  case $mode in
    wrong-codec) codec=libx264; format=matroska; profile=high ;;
    wrong-container) format=ivf ;;
    mono) channels=1 ;;
    silence) audio='anullsrc=r=48000:cl=stereo:d=1' ;;
    wrong-size) size=320x180 ;;
    wrong-duration) duration=0.5; audio='sine=frequency=523:sample_rate=48000:duration=0.5' ;;
  esac
  if [[ $format == ivf ]]; then
    ffmpeg -nostdin -v error -y -f lavfi -i "testsrc2=s=$size:r=12:d=$duration" \
      -an -c:v libvpx-vp9 -profile:v 0 -pix_fmt yuv420p -f ivf "$file"
  else
    ffmpeg -nostdin -v error -y -f lavfi -i "testsrc2=s=$size:r=12:d=$duration" -f lavfi -i "$audio" \
      -t "$duration" -c:v "$codec" -profile:v "$profile" -pix_fmt yuv420p -c:a libopus -ar 48000 -ac "$channels" \
      -f "$format" "$file"
  fi
}

corrupt_codec_artifact() {
  local file=$1 size truncated="$1.truncated"
  size=$(wc -c <"$file" | tr -d ' ')
  dd if="$file" of="$truncated" bs=1 count=$((size * 2 / 3)) 2>/dev/null
  mv "$truncated" "$file"
}

make_codec_fixture() {
  local dir="$1/delivery-codec-matrix" author preview plan
  mkdir -p "$dir/project" "$dir/plans/preview" "$dir/rendered"
  printf 'codec matrix fixture\n' >"$dir/project/main.veac"
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_codec-matrix)
  write_codec_canonical "$author" 640 360 24; write_codec_canonical "$preview" 480 270 12
  jq 'first(.project.render_configs[] | select(.id == "out_codec-matrix")) as $out |
    {output:{id:"pout_codec-matrix",render_config_id:$out.id,sequence_id:$out.sequence_id,
      raster:$out.raster,deliverables:$out.deliverables},
     sequences:[{id:"seq_main",duration:{timescale:1000,value:1000}}]}' "$preview" >"$plan"
  make_hevc_artifact "$dir/rendered/hevc.mov"
  make_vp9_artifact "$dir/rendered/vp9.webm"
}
