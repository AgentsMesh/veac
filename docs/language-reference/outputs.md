# Delivery 与 Deliverable

Project-owned `Delivery` 选择一个 Sequence handle、optional raster contract 和一组 typed
`Deliverable`。每种 artifact 是不同 constructor，不是 `kind` 字符串加 property map。

## Video

```veac,fragment
let picture = video_output(
  video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
  video_crf(20), gop_auto(), b_frames_auto(),
  video_profile_present(profile_h264_high()), video_level_auto()
);
let master = deliverable_video(
  identifier("master"), delivery_file("master.mp4"),
  video_delivery(
    container_mp4(), picture,
    embedded_audio_present(audio_output(audio_aac(), 48000, 2)),
    true, pass_single(), hardware_software()
  )
);
delivery(
  identifier("release"), timeline,
  raster_settings(canvas(1920px, 1080px), frame_rate(30, 1), caption_burn_in()),
  [master]
)
```

Container、video/audio codec、pixel/alpha/color、rate control、GOP、B-frame、profile、level、pass 与
hardware policy 都是闭合 value。显式 hardware backend 在 device/upload binding 未建模时 fail closed。

## Typed Artifact Set

公开 artifact family 及 constructor：

| artifact | constructor |
| --- | --- |
| video | `deliverable_video` |
| image sequence | `deliverable_image_frames` |
| caption sidecar | `deliverable_caption_sidecar` |
| audio stem | `deliverable_audio_stem` |
| scope | `deliverable_scope` |
| audio file | `deliverable_mp3` |
| animated image | `deliverable_gif` |
| still image | `deliverable_still` |
| adaptive package | `deliverable_hls` |

```veac,fragment
deliverable_image_frames(
  identifier("frames"), delivery_image_sequence("frame-%04d.png"), image_png(), 1
)
deliverable_caption_sidecar(
  identifier("transcript"), delivery_file("captions.vtt"),
  caption_webvtt(), [captions]
)
deliverable_audio_stem(
  identifier("master-audio"), delivery_file("master.wav"), stem_wav(),
  audio_output(audio_pcm_s24le(), 48000, 2), mix_master()
)
deliverable_scope(
  identifier("waveform"), delivery_file("waveform.png"),
  scope_waveform(), 1s, canvas(1280px, 720px), image_png()
)
```

Image format 是 PNG/JPEG/TIFF/EXR；caption 是 SRT/WebVTT/ASS；stem 是 WAV/FLAC 并选择 master、
Layer 或 bus mix。Scope 是 waveform/vectorscope/histogram。Target path 经过 leaf/package/pattern
validation，image sequence pattern 只能有一个 frame placeholder。

## MP3、GIF、Still 与 HLS

```veac,fragment
deliverable_mp3(
  identifier("podcast"), delivery_file("podcast.mp3"),
  mix_master(), 192000, 48000, channel_stereo()
)
deliverable_gif(
  identifier("preview"), delivery_file("preview.gif"),
  gif_forever(), gif_dither_sierra2()
)
deliverable_still(
  identifier("cover"), delivery_file("cover.png"), 2s, image_png()
)
deliverable_hls(
  identifier("stream"), delivery_package("stream"), 2s,
  hls_audio_aac(mix_master(), 128000, 48000, channel_stereo()),
  [mobile, hd]
)
```

HLS rendition 使用 `hls_rendition` 指定 canvas、target/max/buffer bitrate、profile、level、color 和
B-frame policy。segment duration 是 1s..60s；rendition ID/canvas 唯一，canonical output 按 ID 排序。

完整九类交付示例见
[`examples/delivery-formats/main.veac`](../../examples/delivery-formats/main.veac) 与
[`outputs.veac`](../../examples/delivery-formats/outputs.veac)。canonical envelope schema v10 使用 strict
tagged variants；planner/backend 只消费验证后的 delivery model。
