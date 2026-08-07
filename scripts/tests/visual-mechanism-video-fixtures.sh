#!/usr/bin/env bash

make_color_grade_video() {
  ffmpeg -v error -y -f lavfi -i 'color=c=black:s=480x270:r=12:d=4' -vf \
    "drawbox=x=0:y=0:w=iw:h=ih:color=0x33415c:t=fill:enable='lt(t,2)',
     drawbox=x=240:y=0:w=240:h=ih:color=0x7d8597:t=fill:enable='lt(t,2)',
     drawbox=x=0:y=0:w=iw:h=ih:color=0x70504a:t=fill:enable='gte(t,2)',
     drawbox=x=240:y=0:w=240:h=ih:color=0xc49b82:t=fill:enable='gte(t,2)'" \
    -c:v libx264 -pix_fmt yuv420p "$1"
}

make_blend_modes_video() {
  local path=$1 filter= index end
  local left=(d8c8b8 181020 8f3f50 201020 c06080 f0a070 100810 b03050 604050 1020d0 3060c0 503040)
  local right=(ffffff 504050 206f9f 604050 90d0e0 ffffff 503040 40a0d0 90a0b0 f0d010 d09040 6090b0)
  for index in {0..11}; do
    end=$(awk -v i="$index" 'BEGIN { print i+.999 }')
    filter+="drawbox=x=0:y=0:w=iw:h=ih:color=0x${left[index]}:t=fill:enable='between(t,$index,$end)',"
    filter+="drawbox=x=240:y=0:w=240:h=ih:color=0x${right[index]}:t=fill:enable='between(t,$index,$end)',"
  done
  ffmpeg -v error -y -f lavfi -i 'color=c=black:s=480x270:r=12:d=12' \
    -vf "${filter}null" -c:v libx264 -pix_fmt yuv420p "$path"
}

make_apply_scopes_video() {
  ffmpeg -v error -y \
    -f lavfi -i 'color=c=0x264653:s=480x270:r=12:d=2,drawbox=x=132:y=75:w=216:h=120:color=0x70bfc0:t=fill,boxblur=12' \
    -f lavfi -i 'color=c=0x264653:s=480x270:r=12:d=2,drawbox=x=190:y=95:w=100:h=80:color=0xff7090:t=fill' \
    -f lavfi -i 'color=c=0x10272c:s=480x270:r=12:d=2,drawbox=x=190:y=95:w=100:h=80:color=0x7d2338:t=fill' \
    -filter_complex '[0:v][1:v][2:v]concat=n=3:v=1:a=0,format=yuv420p[v]' \
    -map '[v]' -c:v libx264 "$1"
}

make_apply_scopes_hard_edge_video() {
  ffmpeg -v error -y \
    -f lavfi -i 'color=c=0x264653:s=480x270:r=12:d=2,drawbox=x=132:y=75:w=216:h=120:color=0x70bfc0:t=fill' \
    -f lavfi -i 'color=c=0x264653:s=480x270:r=12:d=2,drawbox=x=190:y=95:w=100:h=80:color=0xff7090:t=fill' \
    -f lavfi -i 'color=c=0x10272c:s=480x270:r=12:d=2,drawbox=x=190:y=95:w=100:h=80:color=0x7d2338:t=fill' \
    -filter_complex '[0:v][1:v][2:v]concat=n=3:v=1:a=0,format=yuv420p[v]' \
    -map '[v]' -c:v libx264 "$1"
}

make_quarter_footprint_video() {
  ffmpeg -v error -y -i "$1" \
    -vf 'scale=240:135,pad=480:270:0:0:color=0x101820' \
    -c:v libx264 -pix_fmt yuv420p "$2"
}

make_apply_scopes_misplaced_video() {
  ffmpeg -v error -y \
    -f lavfi -i 'color=c=0x264653:s=480x270:r=12:d=2,drawbox=x=0:y=0:w=96:h=96:color=0xbe496a:t=fill,drawbox=x=132:y=75:w=216:h=120:color=0x70bfc0:t=fill,boxblur=12' \
    -f lavfi -i 'color=c=0x264653:s=480x270:r=12:d=2,drawbox=x=0:y=0:w=96:h=96:color=0xff7090:t=fill' \
    -f lavfi -i 'color=c=0x10272c:s=480x270:r=12:d=2,drawbox=x=190:y=95:w=100:h=80:color=0x7d2338:t=fill' \
    -filter_complex '[0:v][1:v][2:v]concat=n=3:v=1:a=0,format=yuv420p[v]' \
    -map '[v]' -c:v libx264 "$1"
}

make_label_only_video() {
  local path=$1 stages=$2 duration=$3 total index start end filter=
  total=$(awk -v count="$stages" -v duration="$duration" 'BEGIN { print count*duration }')
  for ((index=0; index<stages; index++)); do
    start=$(awk -v i="$index" -v d="$duration" 'BEGIN { print i*d }')
    end=$(awk -v s="$start" -v d="$duration" 'BEGIN { print s+d-.001 }')
    filter+="drawbox=x=40:y=215:w=$((80+index*8)):h=28:color=0xffee40:t=fill:enable='between(t,$start,$end)',"
  done
  ffmpeg -v error -y -f lavfi -i "color=c=0x405060:s=480x270:r=12:d=$total" \
    -vf "drawbox=x=240:y=0:w=240:h=ih:color=0x708090:t=fill,${filter}null" \
    -c:v libx264 -pix_fmt yuv420p "$path"
}
