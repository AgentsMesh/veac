/// Video visual effect filter methods: blur, opacity, vignette, grain, sharpen, deshake.
use super::FilterGraph;

impl FilterGraph {
    /// Apply Gaussian blur. `strength` is 0-100, mapped to boxblur radius.
    pub fn add_blur(&mut self, input: &str, strength: f64) -> String {
        let out = self.next_label("bl");
        let r = (strength / 5.0).clamp(1.0, 20.0) as u32;
        let expr = format!("boxblur={r}:{r}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply opacity via colorchannelmixer alpha channel.
    pub fn add_opacity(&mut self, input: &str, opacity: f64) -> String {
        let out = self.next_label("opc");
        let expr = format!("format=rgba,colorchannelmixer=aa={opacity}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply vignette effect. `strength` is 0-1.
    pub fn add_vignette(&mut self, input: &str, strength: f64) -> String {
        let out = self.next_label("vig");
        let angle = std::f64::consts::PI / 5.0
            + strength * (std::f64::consts::PI / 2.0 - std::f64::consts::PI / 5.0);
        let expr = format!("vignette=angle={angle}");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply film grain via noise filter. `amount` is 0-1.
    pub fn add_grain(&mut self, input: &str, amount: f64) -> String {
        let out = self.next_label("grn");
        let strength = (amount * 100.0) as u32;
        let expr = format!("noise=alls={strength}:allf=t");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply sharpening via unsharp filter. `amount` > 0.
    pub fn add_sharpen(&mut self, input: &str, amount: f64) -> String {
        let out = self.next_label("shp");
        let expr = format!("unsharp=5:5:{amount}:5:5:0");
        self.add(vec![input.to_string()], &expr, vec![out.clone()]);
        out
    }

    /// Apply video stabilization via deshake (single-pass).
    pub fn add_deshake(&mut self, input: &str) -> String {
        let out = self.next_label("stb");
        self.add(vec![input.to_string()], "deshake", vec![out.clone()]);
        out
    }
}
