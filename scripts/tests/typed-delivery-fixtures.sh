#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout-fixtures.sh"

make_hls_rendition() {
  local master=$1 root=$2 id=$3 width=$4 height=$5 bitrate=$6 maxrate=$7 buffer=$8
  ffmpeg -nostdin -v error -y -i "$master" -map 0:v:0 -map 0:a:0 -t 3 \
    -vf "scale=${width}:${height}" -c:v libx264 -pix_fmt yuv420p \
    -b:v "$bitrate" -maxrate "$maxrate" -bufsize "$buffer" \
    -g 12 -keyint_min 12 -sc_threshold 0 -force_key_frames 'expr:gte(t,n_forced*1)' \
    -c:a aac -b:a 128k -ar 48000 -ac 2 -f hls -hls_segment_type mpegts \
    -hls_time 1 -hls_list_size 0 -hls_playlist_type vod -hls_flags independent_segments \
    -start_number 0 -hls_segment_filename "$root/segment-${id}-%06d.ts" \
    "$root/rendition-${id}.m3u8"
}

make_hls_master() {
  local root=$1
  {
    printf '%s\n' '#EXTM3U' '#EXT-X-VERSION:6'
    printf '%s\n' '#EXT-X-STREAM-INF:BANDWIDTH=3500000,RESOLUTION=1280x720,CODECS="avc1.64001f,mp4a.40.2"'
    printf '%s\n' 'rendition-rnd_hd.m3u8'
    printf '%s\n' '#EXT-X-STREAM-INF:BANDWIDTH=1200000,RESOLUTION=640x360,CODECS="avc1.64001e,mp4a.40.2"'
    printf '%s\n' 'rendition-rnd_mobile.m3u8'
  } >"$root/master.m3u8"
}

make_typed_delivery_artifacts() {
  local rendered="$1/rendered" stream="$1/rendered/stream"
  ffmpeg -nostdin -v error -y -i "$rendered/master.wav" -map 0:a:0 \
    -c:a libmp3lame -b:a 192k -ar 48000 -ac 2 "$rendered/podcast.mp3"
  ffmpeg -nostdin -v error -y -framerate 12 -start_number 1 \
    -i "$rendered/frame-%04d.png" -loop 0 "$rendered/loop-preview.gif"
  cp "$rendered/frame-0025.png" "$rendered/cover.png"
  mkdir -p "$stream"
  make_hls_rendition "$rendered/master.mp4" "$stream" rnd_hd 1280 720 3000k 3210k 6000k
  make_hls_rendition "$rendered/master.mp4" "$stream" rnd_mobile 640 360 1000k 1100k 2000k
  make_hls_master "$stream"
}

write_typed_delivery_project() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"sequences":[{"id":"seq_main","tracks":[{"clips":[
{"id":"itm_burned-label","source":{"type":"caption","text":"类型化交付输出"}},
{"id":"itm_traceable-label","source":{"type":"caption","text":"可追踪的字幕伴随文件"}}]}]}],
"render_configs":[{"id":"out_master","sequence_id":"seq_main",
"raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1},"captions":"burn_in"},
"deliverables":[
{"id":"dlv_cover","target":{"type":"file","name":"cover.png"},"kind":{"type":"still_image","settings":{"frame":{"mode":"containing","at":{"timescale":1000,"value":2000}},"encoding":"png"}}},
{"id":"dlv_frames","target":{"type":"image_sequence","pattern":"frame-%04d.png"},"kind":{"type":"image_sequence","settings":{"format":"png","start_number":1}}},
{"id":"dlv_loop-preview","target":{"type":"file","name":"loop-preview.gif"},"kind":{"type":"animated_image","settings":{"type":"gif","settings":{"playback":{"mode":"forever"},"dither":"sierra2"}}}},
{"id":"dlv_master","target":{"type":"file","name":"master.mp4"},"kind":{"type":"video","settings":{"container":"mp4","video":{"codec":"h264","pixel_format":"yuv420p","alpha":"opaque","color_space":null,"rate_control":{"type":"crf","value":23},"gop_size":null,"b_frames":null,"profile":null,"level":null},"audio":{"codec":"aac","sample_rate":48000,"channels":2},"pass_mode":"single","hardware":{"type":"software"},"optimize_for_streaming":true}}},
{"id":"dlv_master-audio","target":{"type":"file","name":"master.wav"},"kind":{"type":"audio_stem","settings":{"source":{"type":"master"},"format":"wav","audio":{"codec":"pcm_s24_le","sample_rate":48000,"channels":2}}}},
{"id":"dlv_podcast","target":{"type":"file","name":"podcast.mp3"},"kind":{"type":"audio_file","settings":{"source":{"type":"master"},"encoding":{"type":"mp3","settings":{"bitrate_bps":192000,"sample_rate_hz":48000,"channel_layout":"stereo"}}}}},
{"id":"dlv_stream","target":{"type":"package","name":"stream"},"kind":{"type":"adaptive_package","settings":{"type":"hls","settings":{"segment_duration":{"timescale":1000,"value":1000},"audio":{"source":{"type":"master"},"encoding":{"type":"aac","settings":{"bitrate_bps":128000,"sample_rate_hz":48000,"channel_layout":"stereo"}}},"renditions":[
{"id":"rnd_hd","raster":{"width":1280,"height":720},"encoding":{"type":"h264","settings":{"rate_control":{"target_bps":3000000,"max_bps":3210000,"buffer_size_bits":6000000},"b_frames":null,"profile":null,"level":null,"color_space":null}}},
{"id":"rnd_mobile","raster":{"width":640,"height":360},"encoding":{"type":"h264","settings":{"rate_control":{"target_bps":1000000,"max_bps":1100000,"buffer_size_bits":2000000},"b_frames":null,"profile":null,"level":null,"color_space":null}}}]}}}},
{"id":"dlv_transcript","target":{"type":"file","name":"captions.vtt"},"kind":{"type":"caption_sidecar","settings":{"format":"web_vtt","track_ids":["trk_subtitles"]}}},
{"id":"dlv_video-waveform","target":{"type":"file","name":"video-waveform.png"},"kind":{"type":"scope","settings":{"scope":"waveform","at":{"timescale":1000,"value":1000},"width":1280,"height":720,"format":"png"}}}
]}]}}
JSON
}

write_typed_delivery_plan() {
  local canonical=$1 plan=$2
  jq '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    {output:{id:"pout_master",render_config_id:$config.id,
      sequence_id:$config.sequence_id,raster:$config.raster,deliverables:$config.deliverables},
     sequences:[{id:$config.sequence_id,duration:{timescale:1000,value:3000}}]}
  ' "$canonical" >"$plan"
}

make_delivery() {
  local root=$1 dir="$1/delivery-formats/rendered" project="$1/delivery-formats/project"
  prepare_preview_fixture_dirs "$root/delivery-formats"
  ffmpeg -nostdin -v error -y -f lavfi -i 'color=c=#0d1b2a:s=480x270:r=12:d=3' \
    -f lavfi -i 'sine=frequency=220:sample_rate=48000:duration=3' \
    -vf "drawbox=x=0:y=0:w=iw:h=ih:color=0xc43a69:t=fill:enable='gte(t,1.5)',drawbox=x=170:y=115:w=140:h=40:color=white:t=fill:enable='lt(t,1.5)',drawbox=x=130:y=115:w=220:h=40:color=white:t=fill:enable='gte(t,1.5)'" \
    -t 3 -c:v libx264 -pix_fmt yuv420p -c:a aac -ac 2 -movflags +faststart "$dir/master.mp4"
  ffmpeg -nostdin -v error -y -i "$dir/master.mp4" -map 0:v:0 -frames:v 36 \
    -start_number 1 "$dir/frame-%04d.png"
  ffmpeg -nostdin -v error -y -f lavfi \
    -i 'sine=frequency=220:sample_rate=48000:duration=3' -ac 2 -c:a pcm_s24le "$dir/master.wav"
  ffmpeg -nostdin -v error -y -ss 1 -i "$dir/master.mp4" \
    -vf 'format=yuv444p,waveform=mode=column:components=7:display=overlay,scale=1280:720' \
    -frames:v 1 "$dir/video-waveform.png"
  cat >"$dir/captions.vtt" <<'VTT'
WEBVTT

00:00:00.000 --> 00:00:01.500
类型化交付输出

00:00:01.500 --> 00:00:03.000
可追踪的字幕伴随文件
VTT
  make_typed_delivery_artifacts "$root/delivery-formats"
  write_typed_delivery_project "$project/project.veac.json"
  mirror_fixture_preview_canonical "$root/delivery-formats"
  write_typed_delivery_plan "$project/project.preview.veac.json" \
    "$root/delivery-formats/plans/preview/out_master.json"
}
