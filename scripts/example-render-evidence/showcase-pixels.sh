#!/usr/bin/env bash

region_rgb() {
  local video=$1 time=$2 crop=$3
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "crop=$crop,scale=1:1:flags=area,format=rgb24" -f rawvideo - 2>/dev/null |
    od -An -tu1 -N3 | awk '{ print $1, $2, $3 }'
}

region_color_count() {
  local video=$1 time=$2 crop=$3 mode=$4
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "crop=$crop,format=rgb24" -f rawvideo - 2>/dev/null |
    od -An -v -tu1 |
    awk -v mode="$mode" '
      {
        for (i = 1; i <= NF; i++) {
          channel = bytes % 3
          if (channel == 0) r = $i
          else if (channel == 1) g = $i
          else {
            hit = mode == "white" && r > 185 && g > 185 && $i > 185
            if (mode == "cyan") hit = r < 160 && g > 95 && $i > 115 && $i > r + 20
            if (mode == "green") hit = g > 105 && g > r + 25 && g > $i + 20
            if (mode == "rose") hit = r > 135 && r > g + 35 && r > $i + 10
            if (mode == "navy") hit = r < 55 && g < 75 && $i > 65 && $i > g + 25
            if (mode == "black") hit = r < 35 && g < 35 && $i < 35
            if (hit) count++
          }
          bytes++
        }
      }
      END { print count + 0 }'
}

region_yuv_spread() {
  local video=$1 time=$2 crop=$3
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "crop=$crop,format=yuv444p,signalstats,metadata=print:file=-" -f null - 2>/dev/null |
    awk -F= '
      /YMIN=/ { ymin=$2 } /YMAX=/ { ymax=$2 }
      /UMIN=/ { umin=$2 } /UMAX=/ { umax=$2 }
      /VMIN=/ { vmin=$2 } /VMAX=/ { vmax=$2 }
      END { if (ymax != "") print ymax-ymin, umax-umin, vmax-vmin }'
}

frame_edge_avg() {
  local video=$1 time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf 'edgedetect=low=0.02:high=0.12,signalstats,metadata=print:file=-' \
    -f null - 2>/dev/null |
    awk -F= '/lavfi.signalstats.YAVG=/ { print $2; exit }'
}

frame_sharpen_halo_contrast() {
  local video=$1 time=$2 near far
  near=$(region_yavg "$video" "$time" '2:ih/2:2*iw/5-2:ih/4')
  far=$(region_yavg "$video" "$time" '2:ih/2:2*iw/5-6:ih/4')
  awk -v near="$near" -v far="$far" 'BEGIN { print far-near }'
}

frame_difference_avg() {
  local video=$1 first=$2 second=$3
  ffmpeg -v error -ss "$first" -i "$video" -ss "$second" -i "$video" \
    -filter_complex \
    '[0:v]scale=160:90:flags=area,setpts=PTS-STARTPTS[a];
     [1:v]scale=160:90:flags=area,setpts=PTS-STARTPTS[b];
     [a][b]blend=all_mode=difference,signalstats,metadata=print:file=-' \
    -frames:v 1 -f null - 2>/dev/null |
    awk -F= '/lavfi.signalstats.YAVG=/ { print $2; exit }'
}

frame_green_excess_avg() {
  local video=$1 time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf 'scale=160:90:flags=area,format=rgb24' -f rawvideo - 2>/dev/null |
    od -An -v -tu1 | awk '
      {
        for (i = 1; i <= NF; i++) {
          channel = bytes % 3
          if (channel == 0) r = $i
          else if (channel == 1) g = $i
          else {
            excess = g - (r > $i ? r : $i)
            if (excess > 0) total += excess
            pixels++
          }
          bytes++
        }
      }
      END { if (pixels) printf "%.4f\n", total/pixels }'
}

frame_saturation_avg() {
  local video=$1 time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf 'signalstats,metadata=print:file=-' -f null - 2>/dev/null |
    awk -F= '/lavfi.signalstats.SATAVG=/ { print $2; exit }'
}

frame_axis_difference_avg() {
  local video=$1 time=$2 width=${3:-160} height=${4:-90}
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "scale=$width:$height:flags=area,format=gray" -pix_fmt gray -f rawvideo - 2>/dev/null |
    od -An -v -tu1 |
    awk -v width="$width" -v expected="$((width * height))" '
      {
        for (i = 1; i <= NF; i++) {
          value = $i
          x = pixel % width
          if (x > 0) {
            delta = value - left
            x_total += delta < 0 ? -delta : delta
            x_count++
          }
          if (pixel >= width) {
            delta = value - above[x]
            y_total += delta < 0 ? -delta : delta
            y_count++
          }
          above[x] = value
          left = value
          pixel++
        }
      }
      END {
        if (pixel != expected || !x_count || !y_count) exit 1
        printf "%.4f %.4f\n", x_total/x_count, y_total/y_count
      }'
}
