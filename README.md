# VEAC — Video Editing as Code

![CI](https://github.com/AgentsMesh/veac/actions/workflows/ci.yml/badge.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

**Let AI agents edit videos by writing code, not clicking buttons.**

VEAC is a declarative DSL (Domain-Specific Language) designed to bridge the gap between AI agents and video editing. AI agents are exceptionally good at reading and writing files — but they can't operate GUI-based video editors. VEAC turns video editing into a file-processing task: agents write `.veac` source files, and the VEAC compiler transforms them into FFmpeg commands to produce the final video.

```
User intent → AI Agent → .veac file → VEAC compiler → FFmpeg → video output
```

By abstracting video editing into a simple, declarative text format, any AI agent with file I/O capabilities can become a video editor — no mouse, no timeline UI, just code.

## Quick Example

```veac
project "hello" {
    resolution = "1920x1080"
    fps = 30
    format = "mp4"
}

asset intro = video("assets/intro.mp4")

timeline main {
    track video {
        clip intro { from = 0s  to = 10s }
    }
}
```

## Features

- 21 video/audio effects (brightness, contrast, speed, zoom, blur, etc.)
- 20 transition types (fade, dissolve, wipe, slide, etc.)
- Text, image, PIP, subtitle overlays
- Variable system with `let` bindings
- Module system with `include` for reuse
- Multiple time formats: seconds (`3.5s`), milliseconds (`500ms`), frames (`84f`), SMPTE timecode
- Built-in formatter (`veac fmt`)
- Batch rendering from CSV templates
- Dry-run mode (`veac plan`) to preview FFmpeg commands

## Installation

### Quick Install (Mac / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | sh
```

Specify version or install directory:

```bash
VEAC_VERSION=v0.1.0 VEAC_INSTALL_DIR=~/.local/bin \
  curl -fsSL https://raw.githubusercontent.com/AgentsMesh/veac/main/install.sh | sh
```

### Build from Source

```bash
# Prerequisites: Rust toolchain 1.70+ and FFmpeg 4.0+
git clone https://github.com/AgentsMesh/veac.git
cd veac
cargo install --path crates/veac-cli
```

## Quick Start

```bash
veac build main.veac -o output.mp4   # Compile and render
veac check main.veac                  # Validate only
veac plan main.veac                   # Show FFmpeg commands (dry run)
veac fmt main.veac                    # Format source
veac probe assets/intro.mp4           # Show media info
veac batch template.veac --params data.csv -o out/  # Batch render
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [CLI Reference](docs/cli-reference.md)
- [Language Reference](docs/language-reference/)
- [Architecture](docs/architecture.md)

## Examples

See the [`examples/`](examples/) directory:

- `hello-world/` — Minimal single-clip project
- `minimal/` — Basic project structure
- `text-overlay/` — Text overlays demo
- `transitions/` — Transition effects
- `image-overlay/` — Image watermark
- `speed-demo/` — Speed control
- `color-grade/` — Color grading effects
- `batch-demo/` — Batch rendering from CSV
- `all-features/` — Comprehensive demo with all capabilities

## Why VEAC?

AI agents excel at file processing — reading, writing, and transforming text. But video editing has traditionally required GUI interaction that agents simply cannot perform. VEAC solves this by turning video editing into a **file-processing problem**:

- **Agent-first design**: The syntax is intentionally simple and non-Turing-complete, making it easy for LLMs to generate correct `.veac` files without hallucinating complex logic
- **Declarative**: Describe *what* the video should be, not *how* to process it — agents describe the desired outcome, VEAC handles the FFmpeg complexity
- **Verifiable**: `veac check` validates the file before rendering, giving agents fast feedback loops without waiting for video output
- **Single source format**: `.veac` files are plain text — agents can read, write, diff, and version-control them with standard file tools

## License

MIT — see [LICENSE](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
