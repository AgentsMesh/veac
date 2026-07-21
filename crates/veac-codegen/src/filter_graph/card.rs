/// Filter helpers for compositing an overlay source as a styled "card":
/// aspect-preserving fit, rounded corners (RGB-preserving), and a soft drop shadow.
/// Also the still-image → timed-stream helper both clips and overlays need.
use veac_lang::ir::{FitMode, Shadow};

use super::FilterGraph;

/// Parse an FFmpeg color (`0xRRGGBB`, `#RRGGBB`, or a name) into `(r, g, b)` bytes.
/// Non-hex names fall back to black — a drop shadow is dark by definition.
fn parse_rgb(color: &str) -> (u8, u8, u8) {
    let hex = color
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return (r, g, b);
        }
    }
    (0, 0, 0)
}

impl FilterGraph {
    /// Scale `input` into a `w`×`h` box honoring `fit`:
    /// - `Fill`: stretch to the box (legacy behavior, may distort).
    /// - `Letterbox` (contain): fit inside preserving aspect (may end up smaller than the box).
    /// - `Crop` (cover): fill the box preserving aspect, cropping the overflow.
    pub fn add_fit_scale(&mut self, input: &str, w: u32, h: u32, fit: FitMode) -> String {
        let out = self.next_label("fit");
        let expr = match fit {
            FitMode::Fill => format!("scale={w}:{h}"),
            FitMode::Letterbox => format!("scale={w}:{h}:force_original_aspect_ratio=decrease"),
            FitMode::Crop => {
                format!("scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h}")
            }
        };
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Round the corners of a stream to `radius` px. Uses `geq` on the alpha plane only —
    /// RGB is copied verbatim (`r(X,Y)`/`g`/`b`), so colors are never touched (a luma-based
    /// approach corrupts them); only the four corner regions get their alpha zeroed.
    /// Commas inside the expression are `\,`-escaped for the filtergraph parser.
    pub fn add_round_corners(&mut self, input: &str, radius: u32) -> String {
        let out = self.next_label("round");
        let r = radius as f64;
        // A pixel is dropped iff it sits in a corner box (|X-W/2| > W/2-r AND |Y-H/2| > H/2-r)
        // and beyond `r` from that corner's arc center. Everywhere else keeps source alpha.
        let alpha = format!(
            "if(gt(abs(X-(W/2))\\,(W/2)-{r})*gt(abs(Y-(H/2))\\,(H/2)-{r})\\,\
if(lte(hypot(abs(X-(W/2))-((W/2)-{r})\\,abs(Y-(H/2))-((H/2)-{r}))\\,{r})\\,alpha(X\\,Y)\\,0)\\,alpha(X\\,Y))"
        );
        let expr = format!("format=rgba,geq=r='r(X\\,Y)':g='g(X\\,Y)':b='b(X\\,Y)':a='{alpha}'");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Split a stream into two identical branches (one for the shadow silhouette, one for the card).
    pub fn add_split(&mut self, input: &str) -> (String, String) {
        let a = self.next_label("spl");
        let b = self.next_label("spl");
        self.add(vec![input.to_string()], "split", vec![a.clone(), b.clone()]);
        (a, b)
    }

    /// Build a soft drop shadow from a card stream: pad (so the blur has room to bleed), recolor
    /// to the shadow color at `opacity`, and gaussian-ish blur the alpha edge. Returns
    /// `(label, pad_px)`; the caller overlays it at `(card_x - pad + dx, card_y - pad + dy)`.
    pub fn add_card_shadow(&mut self, card: &str, sh: &Shadow) -> (String, f64) {
        let out = self.next_label("shadow");
        let pad = (sh.blur * 2.0).ceil();
        let (r, g, b) = parse_rgb(&sh.color);
        let op = sh.opacity;
        let blur = sh.blur;
        let expr = format!(
            "pad=iw+{p2}:ih+{p2}:{pad}:{pad}:color=black@0.0,format=rgba,\
geq=r={r}:g={g}:b={b}:a='alpha(X\\,Y)*{op}',boxblur=luma_radius={blur}:luma_power=1:alpha_radius={blur}:alpha_power=1",
            p2 = pad * 2.0
        );
        self.add(vec![card.to_string()], &expr, vec![out.clone()]);
        (out, pad)
    }

    /// Turn a still image input into a `dur`-second constant-rate stream. A raw image input is a
    /// single frame, so `trim`/`duration` on it collapses; looping it to `dur` at `fps` fixes
    /// image clips on the main track and gives image overlays real frames to fade across.
    pub fn add_image_loop(&mut self, input: &str, dur: f64, fps: u32) -> String {
        let out = self.next_label("imgloop");
        let expr = format!(
            "loop=loop=-1:size=1:start=0,fps={fps},trim=duration={dur},setpts=PTS-STARTPTS"
        );
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }
}
