#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-text-animation-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common pixels showcase-pixels text text-showcase; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done

write_canonical() {
  jq -n '
    def t($value): {timescale:1000,value:$value};
    def animation($unit):
      {keyframes:[{time:t(0),value:0},{time:t(500),value:0.55},
        {time:t(1000),value:1}],type:"keyframes"} as $progress |
      (if $unit=="whole" then {type:"constant",value:1} else $progress end) as $reveal |
      {granularity:$unit,stagger:t(45),reveal:$reveal,
       highlight:{fill:{red:250,green:204,blue:21,alpha:255},progress:$reveal},
       opacity:{keyframes:[{time:t(0),value:0},{time:t(400),value:1}]},
       transform:{position_offset:{keyframes:[{value:{y:{value:28}}},{value:{y:{value:0}}}]},
         scale:{keyframes:[{value:{x:0.75,y:0.75}},{value:{x:1,y:1}}]}}};
    def clip($key;$text;$unit;$start):
      {authorship:{logical_path:["text-animation",$key]},
       record_range:{start:t($start),duration:t(1500)},
       source:{type:"text",text:$text,style:{animation:animation($unit)}}};
    {project:{sequences:[{tracks:[{clips:[
      clip("whole";"整块动画";"whole";0),
      clip("line";"第一行动画\n第二行动画";"line";1500),
      clip("word";"逐词 动画 类型化 关键帧";"word";3000),
      clip("grapheme";"逐字动画：打字机揭示";"grapheme";4500)]}]}]}}
  ' >"$1"
}

make_video() {
  local path=$1 filter=$2 duration=${3:-6}
  ffmpeg -v error -y -f lavfi -i "color=c=black:s=480x270:r=12:d=$duration" \
    -vf "$filter,fps=12" -c:v libx264 -pix_fmt yuv420p "$path"
}

progress_filter() {
  local start
  printf "drawbox=x=230:y=135:w=20:h=10:color=white:t=fill:enable='between(t,.05,.35)',"
  printf "drawbox=x=210:y=120:w=60:h=15:color=white:t=fill:enable='between(t,.4,.8)',"
  printf "drawbox=x=180:y=105:w=120:h=12:color=0xffee40:t=fill:enable='between(t,.85,1.49)',"
  printf "drawbox=x=190:y=132:w=100:h=12:color=0xffee40:t=fill:enable='between(t,.85,1.49)',"
  for start in 1.5 3 4.5; do
    printf "drawbox=x=230:y=135:w=20:h=10:color=0xffee40:t=fill:enable='between(t,%s,%s)'," \
      "$(awk -v s="$start" 'BEGIN{print s+.05}')" "$(awk -v s="$start" 'BEGIN{print s+.35}')"
    printf "drawbox=x=210:y=120:w=60:h=15:color=0xffee40:t=fill:enable='between(t,%s,%s)'," \
      "$(awk -v s="$start" 'BEGIN{print s+.4}')" "$(awk -v s="$start" 'BEGIN{print s+.8}')"
    printf "drawbox=x=180:y=105:w=120:h=12:color=0xffee40:t=fill:enable='between(t,%s,%s)'," \
      "$(awk -v s="$start" 'BEGIN{print s+.85}')" "$(awk -v s="$start" 'BEGIN{print s+1.49}')"
    printf "drawbox=x=190:y=132:w=100:h=12:color=0xffee40:t=fill:enable='between(t,%s,%s)'," \
      "$(awk -v s="$start" 'BEGIN{print s+.85}')" "$(awk -v s="$start" 'BEGIN{print s+1.49}')"
  done
  printf 'null'
}

expect_failure() {
  local root=$1 label=$2
  if (check_text_render_contract text-animation "$root/text-animation" \
      "$root/text-animation/rendered/preview.mp4") >/dev/null 2>&1; then
    fail "$label negative text-animation contract passed"
  fi
}

valid="$TMP/valid/text-animation"
mkdir -p "$valid/project" "$valid/rendered"
write_canonical "$valid/project/project.veac.json"
make_video "$valid/rendered/preview.mp4" "$(progress_filter)"
check_text_render_contract text-animation "$valid" "$valid/rendered/preview.mp4"

mutate_canonical() {
  local name=$1 filter=$2 root="$TMP/$1"
  mkdir -p "$root"
  cp -R "$valid" "$root/text-animation"
  jq "$filter" "$valid/project/project.veac.json" >"$root/value.json"
  mv "$root/value.json" "$root/text-animation/project/project.veac.json"
  expect_failure "$root" "$name"
}

mutate_canonical wrong_clip_duration \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="whole")|.record_range.duration.value)=1400'
mutate_canonical wrong_granularity \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="line")|.source.style.animation.granularity)="whole"'
mutate_canonical wrong_stagger \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="word")|.source.style.animation.stagger.value)=0'
mutate_canonical wrong_highlight_progress \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="grapheme")|.source.style.animation.highlight.progress.keyframes[1].value)=0.2'

for mode in static missing-progress wrong-media-duration; do
  root="$TMP/$mode"
  mkdir -p "$root"
  cp -R "$valid" "$root/text-animation"
  case $mode in
    static)
      make_video "$root/text-animation/rendered/preview.mp4" \
        "drawbox=x=180:y=105:w=120:h=40:color=0xffee40:t=fill" ;;
    missing-progress)
      make_video "$root/text-animation/rendered/preview.mp4" "null" ;;
    wrong-media-duration)
      make_video "$root/text-animation/rendered/preview.mp4" "$(progress_filter)" 5 ;;
  esac
  expect_failure "$root" "$mode"
done

printf 'text animation render evidence contract tests passed\n'
