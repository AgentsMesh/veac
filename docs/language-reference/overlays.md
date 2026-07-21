# Overlays

Overlay items are positioned on top of the video timeline using absolute time coordinates.

## Text Overlay

### Syntax
```veac
track text {
    text "content" {
        at       = 1s
        duration = 4s
        font     = "Arial"
        size     = 64
        color    = #FFFFFF
        position = "center"
    }
}
```

### Properties

| Property | Type | Default | Description |
|---|---|---|---|
| `at` | time | `0s` | When the text appears |
| `duration` | time | `5s` | How long the text is visible |
| `font` | string | `"Arial"` | Font family name |
| `size` | integer | `24` | Font size in pixels |
| `color` | color | `#FFFFFF` | Text color |
| `position` | string | `"center"` | Text position on screen |
| `fade_in` | time | — | Fade-in duration |
| `fade_out` | time | — | Fade-out duration |
| `x` / `y` | float | — | Exact pixel position, overriding `position` per axis |
| `shadow` | bool | `false` | Enable a drop shadow behind the glyphs |
| `shadow_x` / `shadow_y` | float | `2` | Shadow offset in px |
| `shadow_color` | color | `black` | Shadow color |
| `outline` | integer | — | Stroke width in px around the glyphs |
| `outline_color` | color | `black` | Stroke color |

## Image Overlay

### Syntax
```veac
track overlay {
    image asset_name {
        at       = 0s
        duration = 30s
        position = "top-right"
        scale    = 0.5
        opacity  = 0.6
    }
}
```

### Properties

| Property | Type | Default | Range | Description |
|---|---|---|---|---|
| `at` | time | `0s` | ≥ 0 | When the image appears |
| `duration` | time | `5s` | ≥ 0 | How long the image is visible |
| `position` | string | `"top-right"` | 9 positions | Image position on screen |
| `scale` | float | — | 0.0 – 10.0 | Scale factor |
| `opacity` | float | — | 0.0 – 1.0 | Opacity level |
| `width` / `height` | float | — | px | Explicit target box (overrides `scale`) |
| `x` / `y` | float | — | px | Exact pixel position, overriding `position` per axis |
| `fade_in` / `fade_out` | time | — | ≥ 0 | Alpha fade in/out (loops the still to give it frames) |
| `radius` | integer | — | px | Rounded corners — see [Card styling](#card-styling) |
| `fit` | string | `"fill"` | fill/contain/cover | Aspect handling in the box |

## Picture-in-Picture (PIP)

### Syntax
```veac
track overlay {
    pip asset_name {
        from     = 0s
        to       = 60s
        at       = 0s
        duration = 60s
        position = "bottom-right"
        scale    = 0.25
    }
}
```

### Properties

| Property | Type | Default | Description |
|---|---|---|---|
| `from` | time | — | Source video start time |
| `to` | time | — | Source video end time |
| `at` | time | `0s` | When PIP appears on timeline |
| `duration` | time | `5s` | How long PIP is visible |
| `position` | string | `"bottom-right"` | PIP position |
| `scale` | float | `0.25` | Scale factor (fraction of the output size) |
| `width` / `height` | float | — | Explicit box in px (overrides `scale`; needed for a square card on a non-square canvas) |
| `x` / `y` | float | — | Exact pixel position, overriding `position` per axis |
| `margin` / `margin_x` / `margin_y` | float | `0` | Inset from the anchored edge in px |
| `fade_in` / `fade_out` | time | `0` | Alpha fade in/out |
| `zoom_in` / `zoom_out` | time | `0` | Animate full-frame ↔ corner (a shrink-to-corner / grow-to-full transition) |
| `fit` | string | `"fill"` | Aspect handling — see [Card styling](#card-styling) |
| `radius` | integer | — | Rounded corners in px |
| `shadow` | bool | `false` | Drop shadow (the PIP is the card vehicle) |
| `shadow_blur` | float | `24` | Shadow blur radius in px |
| `shadow_opacity` | float | `0.5` | Shadow opacity 0–1 |
| `shadow_x` / `shadow_y` | float | `0` / `18` | Shadow offset in px |
| `shadow_color` | color | `black` | Shadow color |

## Card styling

`fit`, `radius`, and `shadow` turn a raw image/video overlay into a **floating card** — the
staple of app-preview and product promos. They share one grammar across `image` and `pip`
overlays (a `pip` is the vehicle when you want a drop shadow).

- **`fit`** controls aspect when the source doesn't match the target box:
  - `"fill"` (default) — stretch to the box (may distort).
  - `"contain"` — fit inside, preserving aspect (may leave transparent margins).
  - `"cover"` — fill the box, preserving aspect, cropping the overflow.
- **`radius`** rounds the corners. Colors are preserved exactly (only the corner alpha is cut).
- **`shadow`** casts a soft, offset, blurred silhouette beneath the card. Tune with
  `shadow_blur` / `shadow_opacity` / `shadow_x` / `shadow_y` / `shadow_color`.

```veac
// A widget clip composited as a rounded, shadowed card on a background image.
asset bg     = image("bg.png")
asset widget = video("widget.mov")

timeline main {
    track video {
        clip bg { duration = 4s }          // a still on the main track now honors `duration`
    }
    track overlay {
        pip widget {
            at = 0s  duration = 4s
            width = 740  height = 775  y = 900
            fit = "cover"  radius = 56
            shadow = true  shadow_blur = 34  shadow_y = 30
        }
    }
}
```

## Subtitle

### Syntax
```veac
track overlay {
    subtitle "path/to/subs.srt" {}
}
```
- Imports an SRT subtitle file
- Subtitles are rendered onto the video

## Gap

### Syntax
```veac
track video {
    gap { duration = 2s }
}
```
- Inserts a silent, black gap in the timeline
- `duration` is the only property (required)

## Freeze Frame

### Syntax
```veac
track video {
    freeze {
        at       = 5s
        duration = 3s
    }
}
```
- Holds a single frame from the preceding clip
- `at`: the time point in the previous clip to freeze
- `duration`: how long to hold the frame

## Position Values

For pixel-exact placement, set `x` / `y` (in px) on a text, image, or pip overlay — they override
the anchor per axis. Otherwise `position` accepts these 9 anchor values:

| Value | Description |
|---|---|
| `"center"` | Center of the frame |
| `"top"` | Top center |
| `"bottom"` | Bottom center |
| `"left"` | Left center |
| `"right"` | Right center |
| `"top-left"` | Top-left corner |
| `"top-right"` | Top-right corner |
| `"bottom-left"` | Bottom-left corner |
| `"bottom-right"` | Bottom-right corner |
