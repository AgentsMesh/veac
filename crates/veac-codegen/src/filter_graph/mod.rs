/// FFmpeg filter graph builder — core structure and rendering.
mod audio;
mod effects;
mod text;
mod transform;
mod video;

/// A single filter node in the graph (e.g., `[0:v]trim=start=5:end=15[v0]`).
#[derive(Debug, Clone)]
pub struct Filter {
    /// Input stream labels, e.g. `["0:v"]`.
    pub inputs: Vec<String>,
    /// Filter name with parameters, e.g. `trim=start=5:end=15`.
    pub expr: String,
    /// Output stream labels, e.g. `["v0"]`.
    pub outputs: Vec<String>,
}

/// Builder for FFmpeg `-filter_complex` strings.
#[derive(Debug, Clone, Default)]
pub struct FilterGraph {
    filters: Vec<Filter>,
    label_counter: usize,
}

impl FilterGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Allocate a unique stream label like `s0`, `s1`, ...
    pub fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Add a filter with explicit inputs/outputs.
    pub fn add(&mut self, inputs: Vec<String>, expr: &str, outputs: Vec<String>) {
        self.filters.push(Filter {
            inputs,
            expr: expr.to_string(),
            outputs,
        });
    }

    /// Render the entire filter graph to an FFmpeg `-filter_complex` value.
    pub fn render(&self) -> String {
        self.filters
            .iter()
            .map(|f| {
                let ins: String = f.inputs.iter().map(|l| format!("[{l}]")).collect();
                let outs: String = f.outputs.iter().map(|l| format!("[{l}]")).collect();
                format!("{ins}{}{outs}", f.expr)
            })
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_simple_concat() {
        let mut g = FilterGraph::new();
        let v0 = g.add_trim("0:v", Some(5.0), Some(15.0));
        let a0 = g.add_atrim("0:a", Some(5.0), Some(15.0));
        let v1 = g.add_trim("1:v", None, None);
        let a1 = g.add_atrim("1:a", None, None);
        let (vout, aout) = g.add_concat(&[v0, v1], &[a0, a1], 2);
        let rendered = g.render();
        assert!(rendered.contains("concat=n=2:v=1:a=1"));
        assert!(rendered.contains(&format!("[{vout}]")));
        assert!(rendered.contains(&format!("[{aout}]")));
    }

    #[test]
    fn render_drawtext() {
        let mut g = FilterGraph::new();
        let out = g.add_drawtext(
            "0:v",
            "Hello",
            "Arial",
            48,
            "white",
            "(w-text_w)/2",
            "(h-text_h)/2",
            3.0,
            8.0,
        );
        let rendered = g.render();
        assert!(rendered.contains("drawtext=text='Hello'"));
        assert!(rendered.contains("enable='between(t,3,8)'"));
        assert!(rendered.contains(&format!("[{out}]")));
    }

    #[test]
    fn render_xfade() {
        let mut g = FilterGraph::new();
        let out = g.add_xfade("v0", "v1", "fade", 1.0, 9.0);
        let rendered = g.render();
        assert!(rendered.contains("xfade=transition=fade:duration=1:offset=9"));
        assert!(rendered.contains(&format!("[{out}]")));
    }

    #[test]
    fn render_speed_change() {
        let mut g = FilterGraph::new();
        let out = g.add_speed("0:v", 2.0);
        let rendered = g.render();
        assert!(rendered.contains("setpts=PTS/2"));
        assert!(rendered.contains(&format!("[{out}]")));
    }

    #[test]
    fn render_atempo() {
        let mut g = FilterGraph::new();
        let out = g.add_atempo("0:a", 2.0);
        let rendered = g.render();
        assert!(rendered.contains("atempo=2"));
        assert!(rendered.contains(&format!("[{out}]")));
    }

    #[test]
    fn render_eq() {
        let mut g = FilterGraph::new();
        let out = g.add_eq("0:v", Some(0.1), Some(1.2), Some(1.5));
        let rendered = g.render();
        assert!(rendered.contains("eq=brightness=0.1:contrast=1.2:saturation=1.5"));
        assert!(rendered.contains(&format!("[{out}]")));
    }
}
