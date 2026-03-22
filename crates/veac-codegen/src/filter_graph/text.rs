/// Text filter methods: drawtext with optional alpha animation.
use super::FilterGraph;

impl FilterGraph {
    /// Build a drawtext filter for text overlay (legacy, no alpha animation).
    #[allow(clippy::too_many_arguments)]
    pub fn add_drawtext(
        &mut self,
        input_label: &str,
        text: &str,
        font: &str,
        size: u32,
        color: &str,
        x_expr: &str,
        y_expr: &str,
        start_sec: f64,
        end_sec: f64,
    ) -> String {
        self.add_drawtext_with_alpha(
            input_label,
            text,
            font,
            size,
            color,
            x_expr,
            y_expr,
            start_sec,
            end_sec,
            None,
        )
    }

    /// Build a drawtext filter with optional alpha animation expression.
    #[allow(clippy::too_many_arguments)]
    pub fn add_drawtext_with_alpha(
        &mut self,
        input_label: &str,
        text: &str,
        font: &str,
        size: u32,
        color: &str,
        x_expr: &str,
        y_expr: &str,
        start_sec: f64,
        end_sec: f64,
        alpha_expr: Option<&str>,
    ) -> String {
        let out = self.next_label("txt");
        // Escape single quotes in the text content.
        let escaped = text.replace('\'', "'\\''");
        let alpha_part = if let Some(a) = alpha_expr {
            format!(":alpha='{a}'")
        } else {
            String::new()
        };
        let expr = format!(
            "drawtext=text='{escaped}':fontfile={font}:fontsize={size}\
             :fontcolor={color}:x={x_expr}:y={y_expr}\
             :enable='between(t,{start_sec},{end_sec})'{alpha_part}"
        );
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }
}
