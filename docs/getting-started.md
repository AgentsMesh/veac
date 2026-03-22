# Getting Started

This guide walks you through installing VEAC, creating your first project, and rendering a video.

## Prerequisites

### Rust Toolchain (1.70+)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify the installation:

```bash
rustc --version
```

### FFmpeg (4.0+)

**macOS (Homebrew):**

```bash
brew install ffmpeg
```

**Ubuntu / Debian:**

```bash
apt install ffmpeg
```

Verify the installation:

```bash
ffmpeg -version
```

## Installation

Clone the repository and install the CLI:

```bash
git clone https://github.com/AgentsMesh/veac.git
cd veac
cargo install --path crates/veac-cli
```

Confirm that the `veac` command is available:

```bash
veac --help
```

## Your First Project

Create a project directory with an asset:

```
my-project/
├── main.veac
└── assets/
    └── intro.mp4
```

Write a minimal `main.veac` file:

```veac
project "my-first-video" {
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

This project declares a 1920x1080 video at 30 fps, imports a single video asset, and places the first 10 seconds of that asset on a video track.

## Build and Run

Use the following commands from inside your project directory:

**Validate the file:**

```bash
veac check main.veac
```

This parses the file and runs semantic analysis without rendering. Errors are reported with line numbers and suggestions.

**Preview the FFmpeg commands:**

```bash
veac plan main.veac
```

This is a dry-run that prints the FFmpeg commands VEAC would execute, useful for debugging.

**Compile and render the video:**

```bash
veac build main.veac -o output.mp4
```

This compiles the `.veac` file and invokes FFmpeg to produce the final video.

## What's Next

- [CLI Reference](cli-reference.md) -- all available commands and their options
- [Language Reference](language-reference/) -- the full VEAC language specification
- [Examples](../examples/) -- more complex projects demonstrating advanced features
