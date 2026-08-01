# Getting Started

## Prerequisites

- Rust toolchain `1.85.0`
- FFmpeg and ffprobe for planning/rendering and E2E tests
- jq for example catalog/build tooling

```bash
make doctor
make build
```

## Write a Project

Create `main.veac`:

```veac
project intro {
  settings {
    timebase 1/1000;
    canvas 1280px by 720px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }
  entry sequence main;

  sequence main {
    layer visual canvas {
      item background {
        source generated solid { color #18202aff; }
        record { at 0s; duration 4s; }
      }
    }
    layer visual title {
      item greeting {
        source text {
          content "Hello, VEAC";
          style { font family "Inter"; size 72px; fill #ffffffff; }
          layout {
            box-width 1000px; box-height 180px;
            horizontal-align center; vertical-align middle;
          }
        }
        record { at 0s; duration 4s; }
      }
    }
  }

  delivery preview {
    sequence main;
    raster { canvas 1280px by 720px; frame-rate 30fps; captions discard; }
    artifact video preview {
      target file "preview.mp4";
      mux mp4 {
        layout fast-start;
        video h264 {
          pixel-format yuv420p;
          alpha opaque;
          color-space source;
          rate-control crf { value 23; }
          gop automatic;
          b-frames automatic;
          profile automatic;
          level automatic;
        }
        audio none;
        passes single;
        accelerator auto;
      }
    }
  }
}
```

## Check and Format

```bash
cargo run -p veac-cli -- check main.veac
cargo run -p veac-cli -- fmt --check main.veac
cargo run -p veac-cli -- fmt main.veac
```

Parser diagnostics include an authoring code and exact source span. Unknown fields, duplicate singleton fields, unresolved typed references, wrong units, and incomplete variants fail before planning.

## Compile, Plan, and Render

```bash
cargo run -p veac-cli -- compile main.veac --out project.json
cargo run -p veac-cli -- plan project.json --config out_preview --out plan.json
cargo run -p veac-cli -- render project.json --out-dir output
```

Compilation emits canonical project IR schema version 5 and does not probe media
or invoke FFmpeg. Planning resolves media, stream selection, clocks, graph
structure, and artifact compatibility. Rendering executes typed backend tasks.

## Explore Mechanisms

```bash
make check-examples
make build-examples
make serve-examples
```

The ignored `examples-preview/` directory contains canonical projects, resolved plans, render logs, deliverables, and a gallery index for every cataloged example.

## Run Guards

```bash
make fmt-check
make clippy
make test
make e2e
make coverage
```

See the [language reference](language-reference/README.md), [architecture](architecture.md), and [CLI reference](cli-reference.md) for the full contracts.
