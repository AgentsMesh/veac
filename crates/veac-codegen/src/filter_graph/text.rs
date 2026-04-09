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
            None,
        )
    }

    /// Build a drawtext filter with optional alpha animation expression.
    ///
    /// `font` can be either:
    /// - An absolute path to a .ttf/.otf file (used as `fontfile=`)
    /// - A font family name (used as `font=` via fontconfig)
    ///
    /// `box_opts` is an optional background box string (e.g. "box=1:boxcolor=black@0.5:boxborderw=12").
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
        box_opts: Option<&str>,
    ) -> String {
        let out = self.next_label("txt");
        // Escape single quotes in the text content.
        let escaped = text.replace('\'', "'\\''");
        let alpha_part = if let Some(a) = alpha_expr {
            format!(":alpha='{a}'")
        } else {
            String::new()
        };
        let box_part = if let Some(b) = box_opts {
            format!(":{b}")
        } else {
            String::new()
        };
        // Use fontfile= for absolute paths, font= for family names (fontconfig).
        let font_param = if font.contains('/') || font.contains('\\') {
            format!("fontfile='{font}'")
        } else {
            format!("font='{font}'")
        };
        let expr = format!(
            "drawtext=text='{escaped}':{font_param}:fontsize={size}\
             :fontcolor={color}:x={x_expr}:y={y_expr}\
             :enable='between(t,{start_sec},{end_sec})'{alpha_part}{box_part}"
        );
        self.add(vec![input_label.to_string()], &expr, vec![out.clone()]);
        out
    }
}
