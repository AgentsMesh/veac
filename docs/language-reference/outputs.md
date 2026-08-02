# Deliveries And Artifacts

A project-owned `delivery` selects one sequence and owns one or more typed
artifacts. Visual artifacts share its raster contract. Artifact IDs are stable;
the artifact kind selects a closed recipe grammar.

```text
video | image-sequence | caption-sidecar | audio-stem | scope
audio-file | animated-image | still-image | adaptive-package
```

Targets are paired with artifact kinds:

- `target file "name.ext";` for single-file artifacts.
- `target pattern "frame-%04d.png";` for image sequences.
- `target package "stream";` for adaptive packages.

File and package targets are safe leaf names. An image pattern contains exactly
one `%d` or `%0Nd` frame placeholder.

## Video File

`mux` owns the container, streams, layout, pass mode, and accelerator. Each
stream names its closed codec before codec-specific settings.

```veac
delivery release {
  sequence main;
  raster { canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }
  artifact video master {
    target file "master.mp4";
    mux mp4 {
      layout fast-start;
      video h264 {
        pixel-format yuv420p;
        alpha opaque;
        color-space source;
        rate-control crf { value 18; }
        gop automatic;
        b-frames automatic;
        profile h264-high;
        level "4.1";
      }
      audio aac {
        sample-rate 48khz;
        channel-layout stereo;
      }
      passes single;
      accelerator auto;
    }
  }
}
```

Containers are `mp4`, `mov`, `mkv`, `webm`, and `mxf`. Video codecs are `h264`,
`h265`, `vp9`, `av1`, `prores`, and `dnxhr`; audio may be `none` or a compatible
AAC, Opus, FLAC, or PCM recipe. Rate control is `crf`, `average`, `capped`, or
`lossless`. `color-space source`, `gop automatic`, `b-frames automatic`,
`profile automatic`, and `level automatic` preserve explicit backend choices.

## Sequence, Captions, Stem, And Scope

These recipes keep naming, source selection, analysis, and codec choice separate:

```veac
artifact image-sequence frames {
  target pattern "frame-%04d.png";
  numbering from 1;
  encode png;
}

artifact caption-sidecar transcript {
  target file "captions.vtt";
  source caption-tracks { track subtitles; }
  encode web-vtt;
}

artifact audio-stem dialogue {
  target file "dialogue.wav";
  source bus dialogue;
  encode wav {
    sample-format pcm-s24le;
    sample-rate 48khz;
    channel-layout stereo;
  }
}

artifact scope waveform {
  target file "waveform.png";
  analyze waveform;
  frame containing 1s;
  canvas 1280px by 720px;
  encode png;
}
```

Image encoders are PNG, JPEG, TIFF, and EXR. Caption encoders are SRT, WebVTT,
and ASS. A stem selects `master`, `track <id>`, or `bus <id>` and encodes WAV or
FLAC. Analyses are waveform, vectorscope, or histogram.

## MP3, GIF, And Still Images

```veac
artifact audio-file podcast {
  target file "podcast.mp3";
  source master;
  encode mp3 {
    bitrate 192kbps;
    sample-rate 48khz;
    channel-layout stereo;
  }
}

artifact animated-image preview {
  target file "preview.gif";
  encode gif { playback forever; dither sierra2; }
}

artifact still-image cover {
  target file "cover.png";
  frame containing 1s;
  encode png;
}
```

GIF playback is `once`, `forever`, or an integer such as `3times`. Dither is
`bayer`, `floyd-steinberg`, `sierra2`, or `none`. `frame containing` selects the
timeline frame whose interval contains the authored time.

## HLS Package

`segment-duration` 必须在 1 秒到 60 秒之间；下限保证 FFmpeg 能生成有效的整数
`EXT-X-TARGETDURATION` 和非空 master playlist。

```veac
artifact adaptive-package stream {
  target package "stream";
  package hls {
    segment-duration 2s;
    audio {
      source master;
      encode aac {
        bitrate 192kbps;
        sample-rate 48khz;
        channel-layout stereo;
      }
    }
    rendition mobile {
      canvas 640px by 360px;
      encode h264 {
        rate-control capped {
          target 900kbps; max 963kbps; buffer 1800kbit;
        }
        profile main;
        level "3.1";
        color-space source;
        b-frames automatic;
      }
    }
    rendition hd {
      canvas 1280px by 720px;
      encode h264 {
        rate-control capped {
          target 3mbps; max 3210kbps; buffer 6mbit;
        }
        profile high;
        level "4.0";
        color-space source;
        b-frames automatic;
      }
    }
  }
}
```

HLS audio is an explicit recipe or `audio none;`. Renditions require unique IDs
and canvases; lowering gives IDs the `rnd_` prefix and canonical IR orders them
by ID. Segment duration and frame selection use `s`, `ms`, or `us`.

## Units And Canonical Boundary

Bitrates use `bps`, `kbps`, or `mbps`; rate-control buffers use `bit`, `kbit`, or
`mbit`; sample rates use `hz` or `khz`; dimensions use `px`. Unitless values in
these positions are errors. Authoring recipes lower to tagged, unknown-field-
rejecting canonical IR schema version 5 with minimum reader version 5.
