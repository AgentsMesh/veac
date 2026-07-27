# Outputs

Each output declaration is one closed deliverable kind with a typed sequence target and safe file basename:

```text
video | image-sequence | caption-sidecar | audio-stem | scope
```

Video:

```veac
output video master {
  sequence main;
  file-name "master.mp4";
  encoding {
    container mp4;
    optimize-for-streaming true;
    video { codec h264; pixel-format yuv420p; }
    audio { codec aac; sample-rate 48000; channels 2; }
    captions burn-in;
  }
}
```

Other variants:

```veac
output image-sequence frames {
  sequence main;
  file-name "frame-%04d.png";
  encoding { format png; start-number 1; }
}

output caption-sidecar transcript {
  sequence main;
  file-name "captions.vtt";
  encoding { format web-vtt; tracks { track subtitles; } }
}

output audio-stem mix {
  sequence main;
  file-name "mix.wav";
  encoding {
    format wav;
    audio { codec pcm-s24le; sample-rate 48000; channels 2; }
    source master;
  }
}

output scope waveform {
  sequence main;
  file-name "waveform.png";
  encoding { scope waveform; at 1s; width 1280; height 720; format png; }
}
```

Caption track lists are non-empty and unique in authoring, then sorted during lowering without silent deduplication. Image patterns accept exactly one `%d` or `%0Nd` placeholder. Container, codec, pixel format, sample format, extension, and stream compatibility are validated before planning.
