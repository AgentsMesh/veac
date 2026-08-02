# Project and Resources

Declaration heads contain only a stable kind and identifier. Ownership is expressed by blocks.

```veac
project documentary {
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30000/1001fps;
    sample-rate 48000hz;
  }
  entry sequence main;

  resource video interview {
    locator local { path "assets/interview.mov"; }
    identity { sha256 "<64 lowercase hex>"; }
    streams { video auto; audio auto; }
  }

  sequence main { }
}
```

Resource kinds are closed:

```text
video | audio | image | font | lut-1d | lut-3d
```

Video and audio resources declare both stream intents. Each intent is `auto`, `disabled`, or an explicit stream index. Image resources implicitly select their visual stream. Font and LUT resources have no media streams.

Locators are `local` or `remote`. Remote resources require a content identity. Local resource names and artifact target basenames reject traversal, disallowed path separators, controls, and overlong values.

The project `entry` is a typed sequence reference. IDs become canonical IDs with stable prefixes such as `prj_`, `med_`, `seq_`, `trk_`, `itm_`, `fx_`, `rel_`, `mcg_`, and `out_`.

Background is not a project setting. It is visible content and must be expressed as a generated source on a layer.
