/// Video filter methods: trim, speed, eq, xfade, overlay, scale.
use super::FilterGraph;

/// Build a per-frame dimension expression that animates `full → target` over `zi` seconds at the
/// start and `target → full` over `zo` seconds at the end (local time `t`, 0..`dur`). Used by
/// `add_scale_anim` for pip zoom in/out. Commas stay inside the caller's single-quoted `w='...'`.
fn anim_dim(full: f64, target: f64, zi: f64, zo: f64, dur: f64) -> String {
    let mut e = format!("{target}");
    if zo > 0.0 {
        let os = dur - zo;
        e = format!("if(gt(t,{os}),{target}+({full}-{target})*(t-{os})/{zo},{e})");
    }
    if zi > 0.0 {
        e = format!("if(lt(t,{zi}),{full}+({target}-{full})*t/{zi},{e})");
    }
    e
}

impl FilterGraph {
    /// Build a trim + setpts filter chain for a video stream.
    pub fn add_trim(&mut self, input_label: &str, from: Option<f64>, to: Option<f64>) -> String {
        let out = self.next_label("tv");
        let mut parts = Vec::new();
        if let Some(f) = from {
            parts.push(format!("start={f}"));
        }
        if let Some(t) = to {
            parts.push(format!("end={t}"));
        }
        let trim_expr = if parts.is_empty() {
            "trim".to_string()
        } else {
            format!("trim={}", parts.join(":"))
        };
        let expr = format!("{trim_expr},setpts=PTS-STARTPTS");
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Shift a stream's presentation timestamps forward by `offset` seconds.
    /// A trimmed pip source has its PTS reset to 0 (by `add_trim`); when overlaid with
    /// `enable='between(t,at,end)'` it must carry PTS in `[at, end]`, otherwise the overlay
    /// finds no matching frame during that window and freezes on the source's last frame.
    pub fn add_pts_offset(&mut self, input: &str, offset: f64) -> String {
        let out = self.next_label("pts");
        let expr = format!("setpts=PTS+{offset}/TB");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Animated scale for a zooming pip: interpolates size `full → target → full` over local time.
    /// `eval=frame` re-evaluates the size expressions per frame using `t`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_scale_anim(&mut self, input: &str, full_w: f64, full_h: f64, target_w: f64,
                          target_h: f64, zoom_in: f64, zoom_out: f64, dur: f64) -> String {
        let out = self.next_label("sca");
        let w = anim_dim(full_w, target_w, zoom_in, zoom_out, dur);
        let h = anim_dim(full_h, target_h, zoom_in, zoom_out, dur);
        let expr = format!("scale=w='{w}':h='{h}':eval=frame");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Alpha fade in/out for an overlay (pip) source. Promotes the stream to `yuva420p` so it has an
    /// alpha channel, then ramps that channel 0→1 over `fade_in` at the start and 1→0 over `fade_out`
    /// at the end of the local clip time (0..`dur`). The overlay compositor blends the base track
    /// through the ramp, giving a dissolve. Call BEFORE `add_pts_offset` (fade `st` is 0-based local
    /// PTS, matching a freshly-trimmed source). Returns the output label.
    pub fn add_alpha_fade(&mut self, input: &str, fade_in: f64, fade_out: f64, dur: f64) -> String {
        let out = self.next_label("fade");
        let mut parts = vec!["format=yuva420p".to_string()];
        if fade_in > 0.0 {
            parts.push(format!("fade=t=in:st=0:d={fade_in}:alpha=1"));
        }
        if fade_out > 0.0 {
            let st = dur - fade_out;
            parts.push(format!("fade=t=out:st={st}:d={fade_out}:alpha=1"));
        }
        let expr = parts.join(",");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Overlay with per-frame position eval (for a pip whose size animates). Returns the output label.
    pub fn add_overlay_anim(&mut self, video: &str, image: &str, x: &str, y: &str, start: f64, end: f64) -> String {
        let out = self.next_label("ov");
        let expr = format!("overlay=x='{x}':y='{y}':eval=frame:enable='between(t,{start},{end})'");
        self.add(vec![video.to_string(), image.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply speed change via setpts. Returns the output label.
    pub fn add_speed(&mut self, input_label: &str, multiplier: f64) -> String {
        let out = self.next_label("spd");
        let expr = format!("setpts=PTS/{multiplier}");
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply color grading via the eq filter. Returns the output label.
    pub fn add_eq(
        &mut self,
        input_label: &str,
        brightness: Option<f64>,
        contrast: Option<f64>,
        saturation: Option<f64>,
    ) -> String {
        let out = self.next_label("eq");
        let mut parts = Vec::new();
        if let Some(b) = brightness {
            parts.push(format!("brightness={b}"));
        }
        if let Some(c) = contrast {
            parts.push(format!("contrast={c}"));
        }
        if let Some(s) = saturation {
            parts.push(format!("saturation={s}"));
        }
        let expr = format!("eq={}", parts.join(":"));
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Cross-fade between two video streams using xfade. Returns the output label.
    pub fn add_xfade(
        &mut self,
        in1: &str,
        in2: &str,
        transition: &str,
        duration: f64,
        offset: f64,
    ) -> String {
        let out = self.next_label("xf");
        let expr = format!("xfade=transition={transition}:duration={duration}:offset={offset}");
        self.add(
            vec![in1.to_string(), in2.to_string()],
            &expr,
            vec![out.clone()],
        );
        out
    }

    /// Overlay an image stream onto a video stream with time control. Returns the output label.
    pub fn add_overlay(
        &mut self,
        video: &str,
        image: &str,
        x: &str,
        y: &str,
        start: f64,
        end: f64,
    ) -> String {
        let out = self.next_label("ov");
        let expr = format!("overlay=x={x}:y={y}:enable='between(t,{start},{end})'");
        self.add(
            vec![video.to_string(), image.to_string()],
            &expr,
            vec![out.clone()],
        );
        out
    }

    /// Scale an image/video stream. Returns the output label.
    pub fn add_scale(&mut self, input: &str, w: &str, h: &str) -> String {
        let out = self.next_label("sc");
        let expr = format!("scale={w}:{h}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply zoompan animation: normal → zoom in → hold → zoom out.
    /// `peak` is the maximum zoom level (e.g. 2.0), `duration` in seconds.
    /// Optional `pan_x`/`pan_y` (-1.0 to 1.0) offset the focus point.
    #[allow(clippy::too_many_arguments)]
    pub fn add_zoompan(
        &mut self,
        input: &str,
        peak: f64,
        duration: f64,
        width: u32,
        height: u32,
        fps: u32,
        pan_x: Option<f64>,
        pan_y: Option<f64>,
    ) -> String {
        let out = self.next_label("zp");
        let total_frames = (duration * fps as f64) as u64;
        // Phase frames: ramp-up 30%, hold 40%, ramp-down 30%
        let f1 = (total_frames as f64 * 0.30) as u64;
        let f2 = (total_frames as f64 * 0.70) as u64;
        let f3 = total_frames - f2;
        let pm1 = peak - 1.0;
        // Zoom expression using `on` (output frame number):
        //   0..f1      : lerp 1.0 → peak
        //   f1..f2     : hold peak
        //   f2..total  : lerp peak → 1.0
        let z_expr = format!(
            "if(lt(on\\,{f1})\\,1+on/{f1}*{pm1}\\,if(lt(on\\,{f2})\\,{peak}\\,{peak}-(on-{f2})/{f3}*{pm1}))"
        );
        // Apply pan offset: shift the focus point
        let px = pan_x.unwrap_or(0.0);
        let py = pan_y.unwrap_or(0.0);
        let x_expr = if px == 0.0 {
            "iw/2-(iw/zoom/2)".to_string()
        } else {
            // Shift focus: positive = right, negative = left
            format!("iw/2-(iw/zoom/2)+iw*{px}*(1-1/zoom)/2")
        };
        let y_expr = if py == 0.0 {
            "ih/2-(ih/zoom/2)".to_string()
        } else {
            format!("ih/2-(ih/zoom/2)+ih*{py}*(1-1/zoom)/2")
        };
        let expr = format!(
            "zoompan=z='{z_expr}':x='{x_expr}':y='{y_expr}':d=1:s={width}x{height}:fps={fps}"
        );
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Add a concat filter. Returns `(video_out, audio_out)` labels.
    pub fn add_concat(
        &mut self,
        video_inputs: &[String],
        audio_inputs: &[String],
        n: usize,
    ) -> (String, String) {
        let vout = self.next_label("outv");
        let aout = self.next_label("outa");
        let mut inputs = Vec::with_capacity(n * 2);
        for i in 0..n {
            inputs.push(video_inputs[i].clone());
            inputs.push(audio_inputs[i].clone());
        }
        let expr = format!("concat=n={n}:v=1:a=1");
        self.add(inputs, &expr, vec![vout.clone(), aout.clone()]);
        (vout, aout)
    }

    /// Apply tpad for freeze frame (clone last frame).
    pub fn add_tpad(&mut self, input: &str, stop_duration: f64) -> String {
        let out = self.next_label("tp");
        let expr = format!("tpad=stop_mode=clone:stop_duration={stop_duration}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Generate a color source video stream (for gaps).
    pub fn add_color_source(
        &mut self,
        duration: f64,
        width: u32,
        height: u32,
        fps: u32,
        color: &str,
    ) -> String {
        let out = self.next_label("clr");
        let expr = format!("color=c={color}:s={width}x{height}:r={fps}:d={duration}");
        self.add(vec![], &expr, vec![out.clone()]);
        out
    }

    /// Apply padding for letterbox mode.
    pub fn add_pad(&mut self, input: &str, width: u32, height: u32, color: &str) -> String {
        let out = self.next_label("pad");
        let expr = format!("pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color={color}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }
}
