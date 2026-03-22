/// Audio filter methods: atrim, volume, atempo, afade, acrossfade, amix.
use super::FilterGraph;

impl FilterGraph {
    /// Build an atrim + asetpts filter chain for an audio stream.
    pub fn add_atrim(&mut self, input_label: &str, from: Option<f64>, to: Option<f64>) -> String {
        let out = self.next_label("ta");
        let mut parts = Vec::new();
        if let Some(f) = from {
            parts.push(format!("start={f}"));
        }
        if let Some(t) = to {
            parts.push(format!("end={t}"));
        }
        let trim_expr = if parts.is_empty() {
            "atrim".to_string()
        } else {
            format!("atrim={}", parts.join(":"))
        };
        let expr = format!("{trim_expr},asetpts=PTS-STARTPTS");
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Add a volume filter on an audio stream. Returns the output label.
    pub fn add_volume(&mut self, input_label: &str, volume: f64) -> String {
        let out = self.next_label("vol");
        let expr = format!("volume={volume}");
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply tempo change for audio. Chains atempo filters for extreme values.
    pub fn add_atempo(&mut self, input: &str, tempo: f64) -> String {
        let mut current = input.to_string();
        let mut remaining = tempo;

        // atempo filter only supports 0.5..100.0, chain for values outside this range.
        while remaining < 0.5 {
            let out = self.next_label("at");
            self.add(vec![current], "atempo=0.5", vec![out.clone()]);
            current = out;
            remaining /= 0.5;
        }
        while remaining > 100.0 {
            let out = self.next_label("at");
            self.add(vec![current], "atempo=100.0", vec![out.clone()]);
            current = out;
            remaining /= 100.0;
        }

        let out = self.next_label("at");
        let expr = format!("atempo={remaining}");
        self.add(vec![current], &expr, vec![out.clone()]);
        out
    }

    /// Apply audio fade in or out. `fade_type` is "in" or "out".
    pub fn add_afade(&mut self, input: &str, fade_type: &str, start: f64, duration: f64) -> String {
        let out = self.next_label("af");
        let expr = format!("afade=t={fade_type}:st={start}:d={duration}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Cross-fade between two audio streams. Returns the output label.
    pub fn add_acrossfade(&mut self, in1: &str, in2: &str, duration: f64) -> String {
        let out = self.next_label("axf");
        let expr = format!("acrossfade=d={duration}:c1=tri:c2=tri");
        self.add(
            vec![in1.to_string(), in2.to_string()],
            &expr,
            vec![out.clone()],
        );
        out
    }

    /// Mix multiple audio streams. Returns the output label.
    pub fn add_amix(&mut self, inputs: &[String], n: usize) -> String {
        let out = self.next_label("amx");
        let expr = format!("amix=inputs={n}:duration=longest");
        self.add(inputs.to_vec(), &expr, vec![out.clone()]);
        out
    }

    /// Generate a silent audio stream of the given duration. Returns the output label.
    pub fn add_silence(&mut self, duration: f64) -> String {
        let out = self.next_label("sil");
        let expr = format!("aevalsrc=0:s=44100:d={duration}");
        self.add(vec![], &expr, vec![out.clone()]);
        out
    }

    /// Apply audio reverse. Returns the output label.
    pub fn add_areverse(&mut self, input: &str) -> String {
        let out = self.next_label("arev");
        self.add(vec![input.to_string()], "areverse", vec![out.clone()]);
        out
    }

    /// Apply audio loudness normalization.
    pub fn add_loudnorm(&mut self, input: &str) -> String {
        let out = self.next_label("ln");
        self.add(vec![input.to_string()], "loudnorm", vec![out.clone()]);
        out
    }
}
