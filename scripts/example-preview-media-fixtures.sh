#!/usr/bin/env bash

make_fixture_video() {
  local visual=$1
  local frequency=$2
  local output=$3
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i "$visual" \
    -f lavfi -i "sine=frequency=${frequency}:sample_rate=48000:duration=65" \
    -map 0:v:0 -map 1:a:0 -c:v libx264 -preset ultrafast -crf 34 \
    -pix_fmt yuv420p -c:a aac -b:a 64k -shortest "$output"
}

make_fixture_audio() {
  local frequency=$1
  local filter=$2
  local output=$3
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i "sine=frequency=${frequency}:sample_rate=48000:duration=65" \
    -af "$filter" -c:a pcm_s16le "$output"
}

make_fixture_image() {
  local background=$1
  local grid=$2
  local size=$3
  local output=$4
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i "color=c=${background}:size=${size}:duration=1" \
    -vf "drawgrid=width=32:height=32:color=${grid}@0.8:thickness=2" \
    -frames:v 1 -update 1 "$output"
}

write_preview_videos() {
  local directory=$1
  local shaky_visual
  shaky_visual='color=c=0x20262e:size=360x220:rate=30:duration=65'
  shaky_visual+=',drawbox=x=0:y=0:w=60:h=ih:color=0xe63946:t=fill,drawbox=x=60:y=0:w=60:h=ih:color=0x80ed99:t=fill,drawbox=x=120:y=0:w=60:h=ih:color=0xffd166:t=fill'
  shaky_visual+=',drawbox=x=180:y=0:w=60:h=ih:color=0x4361ee:t=fill,drawbox=x=240:y=0:w=60:h=ih:color=0xf72585:t=fill,drawbox=x=300:y=0:w=60:h=ih:color=0x4cc9f0:t=fill'
  shaky_visual+=',format=rgb24,geq=r=if(lt(mod(X\,24)\,2)*lt(mod(Y\,24)\,2)\,255\,r(X\,Y)):g=if(lt(mod(X\,24)\,2)*lt(mod(Y\,24)\,2)\,255\,g(X\,Y)):b=if(lt(mod(X\,24)\,2)*lt(mod(Y\,24)\,2)\,255\,b(X\,Y))'
  shaky_visual+=',drawbox=x=20+mod(t*150\,250):y=125:w=70:h=50:color=0x202020:t=fill,drawbox=x=20+mod(t*150\,250):y=125:w=70:h=50:color=0x606060:t=fill:enable=between(mod(t\,0.8)\,0.2\,0.399)'
  shaky_visual+=',drawbox=x=20+mod(t*150\,250):y=125:w=70:h=50:color=0xa0a0a0:t=fill:enable=between(mod(t\,0.8)\,0.4\,0.599),drawbox=x=20+mod(t*150\,250):y=125:w=70:h=50:color=0xc0c0c0:t=fill:enable=gte(mod(t\,0.8)\,0.6)'
  shaky_visual+=',crop=320:180:x=20+5.5*sin(n*1.7):y=20+4.5*cos(n*1.3)'
  make_fixture_video \
    'testsrc2=size=320x180:rate=12:duration=65,hue=h=t*7:s=1,drawgrid=w=40:h=30:color=white@0.12:t=1,drawbox=x=mod(t*31\,356)-36:y=mod(t*17\,216)-36:w=36:h=36:color=white:t=fill' \
    330 "$directory/motion.mp4"
  make_fixture_video 'color=c=0xC84630:size=320x180:rate=12:duration=65' \
    440 "$directory/red.mp4"
  make_fixture_video 'color=c=0x168AAD:size=320x180:rate=12:duration=65' \
    550 "$directory/green.mp4"
  make_fixture_video 'color=c=0x5A4FCF:size=320x180:rate=12:duration=65' \
    660 "$directory/blue.mp4"
  make_fixture_video \
    'color=c=0xC84630:size=320x180:rate=30:duration=65,drawgrid=w=40:h=40:color=white@0.12:t=1,drawbox=x=0:y=0:w=24:h=180:color=white@0.9:t=fill,drawbox=x=mod(t*233\,356)-36:y=68:w=36:h=44:color=white:t=fill' \
    330 "$directory/multicam-host.mp4"
  make_fixture_video \
    'color=c=0x168AAD:size=320x180:rate=30:duration=65,drawgrid=w=40:h=40:color=0xF4C95D@0.18:t=1,drawbox=x=296:y=0:w=24:h=180:color=0xF4C95D@0.95:t=fill,drawbox=x=320-mod(t*197\,356):y=68:w=36:h=44:color=0xF4C95D:t=fill' \
    550 "$directory/multicam-guest.mp4"
  make_fixture_video "$shaky_visual" 770 "$directory/shaky.mp4"
}

write_preview_audio() {
  local directory=$1
  make_fixture_audio 440 \
    "volume='if(lt(mod(t\,1)\,0.65)\,0.9\,0.03)':eval=frame" "$directory/voice.wav"
  make_fixture_audio 660 'volume=0.65' "$directory/music.wav"
  make_fixture_audio 1200 \
    "volume='if(between(mod(t\,2)\,0.5\,0.9)+between(mod(t\,2)\,1.3\,1.7)\,1.8\,0)':eval=frame" \
    "$directory/key.wav"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'sine=frequency=220:sample_rate=48000:duration=65' \
    -c:a aac -b:a 64k "$directory/audio.m4a"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'sine=frequency=880:sample_rate=48000:duration=65' \
    -af 'volume=0.35' -c:a aac -b:a 96k "$directory/codec-tone.m4a"
}

write_preview_images() {
  local directory=$1
  make_fixture_image 0x20262E 0x4ADE80 320x180 "$directory/background.png"
  make_fixture_image 0xF4C95D 0x20262E 180x180 "$directory/logo.png"
  make_fixture_image 0xF7F7F2 0xC84630 640x160 "$directory/caption.png"
  make_fixture_image 0x168AAD 0xF7F7F2 320x320 "$directory/widget.png"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x2DC264:size=320x180:duration=1' \
    -vf 'drawgrid=w=20:h=20:color=0x17324D@0.55:t=2,drawbox=x=92:y=35:w=136:h=110:color=0xF4C95D:t=fill,drawbox=x=118:y=58:w=84:h=64:color=0xD1495B:t=fill,drawbox=x=159:y=58:w=3:h=64:color=0x17324D:t=fill' \
    -frames:v 1 -update 1 "$directory/effects-plate.png"
}

write_preview_media_fixtures() {
  local directory=$1
  write_preview_videos "$directory"
  write_preview_audio "$directory"
  write_preview_images "$directory"
}
