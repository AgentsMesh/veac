/// Video transform filter methods: crop, rotate, flip.
use super::FilterGraph;

impl FilterGraph {
    /// Apply crop filter. `spec` is in WxH+X+Y format (e.g. "640x480+100+50").
    pub fn add_crop(&mut self, input: &str, spec: &str) -> String {
        let out = self.next_label("cr");
        // Parse WxH+X+Y into crop=w:h:x:y
        let parts: Vec<&str> = spec.splitn(2, '+').collect();
        let (wh, xy) = if parts.len() == 2 {
            let wh_parts: Vec<&str> = parts[0].split('x').collect();
            let xy_parts: Vec<&str> = parts[1].splitn(2, '+').collect();
            if wh_parts.len() == 2 && xy_parts.len() == 2 {
                (
                    format!("{}:{}", wh_parts[0], wh_parts[1]),
                    format!("{}:{}", xy_parts[0], xy_parts[1]),
                )
            } else {
                (spec.to_string(), "0:0".to_string())
            }
        } else {
            (spec.to_string(), "0:0".to_string())
        };
        let expr = format!("crop={wh}:{xy}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply rotation in degrees using the rotate filter.
    pub fn add_rotate(&mut self, input: &str, degrees: f64) -> String {
        let out = self.next_label("rot");
        let radians = degrees * std::f64::consts::PI / 180.0;
        let expr =
            format!("rotate={radians}:fillcolor=black@0:ow=rotw({radians}):oh=roth({radians})");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply flip: horizontal, vertical, or both.
    pub fn add_flip(&mut self, input: &str, mode: &str) -> String {
        let out = self.next_label("fl");
        let expr = match mode {
            "horizontal" => "hflip".to_string(),
            "vertical" => "vflip".to_string(),
            "both" => "hflip,vflip".to_string(),
            _ => "hflip".to_string(),
        };
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply reverse to video stream.
    pub fn add_reverse(&mut self, input: &str) -> String {
        let out = self.next_label("rev");
        self.add(vec![input.to_string()], "reverse", vec![out.clone()]);
        out
    }

    /// Apply chromakey (green/blue screen removal).
    pub fn add_chromakey(&mut self, input: &str, color: &str) -> String {
        let out = self.next_label("ck");
        let hex = match color {
            "green" => "0x00FF00",
            "blue" => "0x0000FF",
            s if s.starts_with('#') => {
                // Already handled by caller, but let's keep it compatible
                return self.add_chromakey_hex(input, s);
            }
            _ => "0x00FF00", // Default to green
        };
        let expr = format!("chromakey={hex}:0.1:0.2");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    fn add_chromakey_hex(&mut self, input: &str, hex: &str) -> String {
        let out = self.next_label("ck");
        let ffmpeg_hex = hex.replacen('#', "0x", 1);
        let expr = format!("chromakey={ffmpeg_hex}:0.1:0.2");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }
}
