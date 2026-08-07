#!/usr/bin/env bash

write_programming_media() {
  local root=$1 file="$1/programming-language/rendered/preview.mp4"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x2b6574:s=480x270:r=12:d=3' \
    -f lavfi -i 'color=c=0x7b315d:s=480x270:r=12:d=3' -filter_complex \
    "[0:v]fade=t=in:st=0:d=0.8[a];[1:v]fade=t=in:st=0:d=0.8[b];[a][b]concat=n=2:v=1:a=0,
     drawbox=x=120:y=238:w=240:h=22:color=0x07131d:t=fill,
     drawbox=x=150:y=244:w=180:h=6:color=white:t=fill[out]" \
    -map '[out]' -c:v libx264 -pix_fmt yuv420p "$file"
}

write_audio_caption_media() {
  local root=$1 file="$1/executable-audio-caption/rendered/preview.mp4"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x183c36:s=480x270:r=12:d=4' \
    -f lavfi -i 'aevalsrc=0.08*sin(2*PI*660*t)*between(t\,1\,3)|0.08*sin(2*PI*660*t)*between(t\,1\,3):s=48000:d=4:c=stereo' \
    -vf "drawbox=x=175:y=238:w=130:h=20:color=0x07131d:t=fill:enable='lt(t,1.5)',
      drawbox=x=175:y=244:w=130:h=5:color=white:t=fill:enable='lt(t,1.5)',
      drawbox=x=130:y=238:w=220:h=20:color=0x07131d:t=fill:enable='between(t,1.5,2.999)',
      drawbox=x=145:y=244:w=190:h=5:color=white:t=fill:enable='between(t,1.5,2.999)',
      drawbox=x=145:y=238:w=190:h=20:color=0x07131d:t=fill:enable='gte(t,3)',
      drawbox=x=160:y=244:w=160:h=5:color=white:t=fill:enable='gte(t,3)'" \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -ar 48000 -ac 2 "$file"
}

write_dissolve_media() {
  local root=$1 file="$1/executable-centered-dissolve/rendered/preview.mp4"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0xef4444:s=480x270:r=12:d=2' \
    -f lavfi -i 'color=c=0x2563eb:s=480x270:r=12:d=2.4' -filter_complex \
    "[0:v][1:v]xfade=transition=fade:duration=0.4:offset=1.6,
      drawbox=x=150:y=225:w=180:h=30:color=0x07131d:t=fill:enable='not(between(t,1.6,1.999))',
      drawbox=x=177:y=234:w=126:h=8:color=white:t=fill:enable='not(between(t,1.6,1.999))',
      drawbox=x=75:y=225:w=330:h=30:color=0x07131d:t=fill:enable='between(t,1.6,1.999)',
      drawbox=x=98:y=234:w=285:h=8:color=white:t=fill:enable='between(t,1.6,1.999)'[out]" \
    -map '[out]' -c:v libx264 -pix_fmt yuv420p "$file"
}

write_local_image_media() {
  local root=$1 dir="$1/executable-local-image" repository
  repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
  cp "$repository/examples/executable-local-image/assets/executable-local-image.png" \
    "$dir/project/assets/executable-local-image.png"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x18384f:s=64x36:r=12:d=3' -vf \
    'drawbox=x=24:y=4:w=16:h=18:color=black:t=fill,
     drawbox=x=40:y=4:w=16:h=18:color=white:t=fill,
     drawbox=x=8:y=25:w=48:h=8:color=0x07131d:t=fill,
     drawbox=x=20:y=27:w=24:h=3:color=white:t=fill' \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

write_text_family_media() {
  local root=$1 file="$1/executable-text-family/rendered/preview.mp4"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x101827:s=480x270:r=12:d=8' -vf \
    "drawbox=x=80:y=115:w=120:h=35:color=0xffe066:t=fill:enable='between(t,0,0.799)',
     drawbox=x=72:y=115:w=336:h=35:color=0xffe066:t=fill:enable='between(t,0.8,1.999)',
     drawbox=x=228:y=65:w=24:h=80:color=0x48d7ca:t=fill:enable='between(t,2,2.799)',
     drawbox=x=228:y=42:w=24:h=186:color=0x48d7ca:t=fill:enable='between(t,2.8,3.999)',
     drawbox=x=172:y=65:w=24:h=80:color=0xffe066:t=fill:enable='between(t,4,4.799)',
     drawbox=x=172:y=42:w=24:h=186:color=0xffe066:t=fill:enable='between(t,4.8,5.999)',
     drawbox=x=100:y=145:w=90:h=14:color=0x48d7ca:t=fill:enable='between(t,6,6.799)',
     drawbox=x=100:y=165:w=280:h=14:color=0x48d7ca:t=fill:enable='between(t,6.8,7.999)',
     drawbox=x=145:y=145:w=190:h=14:color=0xffe066:t=fill:enable='between(t,6.8,7.999)'" \
    -c:v libx264 -pix_fmt yuv420p "$file"
}

write_executable_family_media() {
  local root=$1
  write_programming_media "$root"
  write_audio_caption_media "$root"
  write_dissolve_media "$root"
  write_local_image_media "$root"
  write_text_family_media "$root"
}
