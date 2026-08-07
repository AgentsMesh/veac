#!/usr/bin/env bash

make_workflow_keying_video() {
  local output=$1 mode=${2:-valid} first='#d1495b' stable_source
  [[ $mode != wrong-chroma ]] || first='#2dbf64'
  if [[ $mode == frozen ]]; then
    stable_source='color=c=#456789:s=488x278:r=30:d=2'
  else
    stable_source='testsrc2=s=488x278:r=30:d=2'
  fi
  ffmpeg -v error -y -f lavfi -i "color=c=$first:s=480x270:r=30:d=1" \
    -f lavfi -i 'color=c=#2dbf64:s=480x270:r=30:d=1' \
    -f lavfi -i "$stable_source" -filter_complex "
      [0:v]drawbox=x=100:y=225:w=280:h=12:color=white:t=fill[v0];
      [1:v]drawbox=x=100:y=225:w=280:h=12:color=white:t=fill[v1];
      [2:v]drawgrid=w=36:h=36:t=2:c=white,
        crop=480:270:x='if(eq($([[ $mode == frozen ]] && echo 1 || echo 0),1),0,mod(floor(n/5),7))':
          y='if(eq($([[ $mode == frozen ]] && echo 1 || echo 0),1),0,mod(floor(n/7),4))',
        drawbox=x=100:y=225:w=280:h=12:color=white:t=fill[v2];
      [v0][v1][v2]concat=n=3:v=1:a=0,format=yuv420p[v]" \
    -map '[v]' -c:v libx264 -preset ultrafast -crf 18 -an -movflags +faststart "$output"
}

make_workflow_routing_video() {
  local output=$1 mode=${2:-valid} right=0.945
  [[ $mode != balanced ]] || right=1
  ffmpeg -v error -y -f lavfi -i 'color=c=#07131d:s=480x270:r=30:d=2' \
    -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=2' -filter_complex "
      [0:v]drawbox=x=80:y=220:w=320:h=15:color=white:t=fill,format=yuv420p[v];
      [1:a]afade=t=in:st=0:d=0.15,afade=t=out:st=1.85:d=0.15,
        pan=stereo|c0=1*c0|c1=$right*c0[a]" \
    -map '[v]' -map '[a]' -c:v libx264 -preset ultrafast -crf 18 \
    -c:a aac -b:a 128k -movflags +faststart "$output"
}

make_workflow_edit_video() {
  local output=$1 mode=${2:-valid} second='#db2777'
  [[ $mode != wrong-second ]] || second='#16a34a'
  ffmpeg -v error -y -f lavfi -i 'color=c=#2563eb:s=480x270:r=30:d=2' \
    -f lavfi -i "color=c=$second:s=480x270:r=30:d=2" -filter_complex '
      [0:v]drawbox=x=80:y=220:w=320:h=15:color=white:t=fill[v0];
      [1:v]drawbox=x=80:y=220:w=320:h=15:color=white:t=fill[v1];
      [v0][v1]concat=n=2:v=1:a=0,format=yuv420p[v]' \
    -map '[v]' -c:v libx264 -preset ultrafast -crf 18 -an -movflags +faststart "$output"
}

make_workflow_probe_video() {
  local output=$1 mode=${2:-valid} marker='drawbox=x=180:y=90:w=120:h=30:color=white:t=fill,'
  [[ $mode != missing-marker ]] || marker=
  ffmpeg -v error -y -f lavfi -i 'color=c=#4caf50:s=480x270:r=30:d=3' \
    -vf "${marker}drawbox=x=80:y=220:w=320:h=15:color=white:t=fill,format=yuv420p" \
    -c:v libx264 -preset ultrafast -crf 18 -an -movflags +faststart "$output"
}
