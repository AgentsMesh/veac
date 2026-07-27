use super::Interpolation;

impl Interpolation {
    pub fn evaluate(&self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Hold => 0.0,
            Self::Linear => progress,
            Self::EaseIn => progress * progress,
            Self::EaseOut => 1.0 - (1.0 - progress).powi(2),
            Self::EaseInOut => progress * progress * (3.0 - 2.0 * progress),
            Self::CubicBezier { .. } => {
                let controls = self.cubic_controls().expect("cubic interpolation");
                let parameter = parameter_for_x(progress, controls[0], controls[2]);
                cubic(parameter, controls[1], controls[3])
            }
        }
    }

    pub(crate) fn cubic_controls(&self) -> Option<[f64; 4]> {
        match self {
            Self::Linear => Some([1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0]),
            Self::EaseIn => Some([1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0 / 3.0]),
            Self::EaseOut => Some([1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 1.0]),
            Self::EaseInOut => Some([1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0]),
            Self::CubicBezier { x1, y1, x2, y2 } => Some([*x1, *y1, *x2, *y2]),
            Self::Hold => None,
        }
    }

    pub(crate) fn parameter_for_x(&self, x: f64) -> Option<f64> {
        let controls = self.cubic_controls()?;
        Some(parameter_for_x(x, controls[0], controls[2]))
    }
}

fn parameter_for_x(x: f64, x1: f64, x2: f64) -> f64 {
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..60 {
        let middle = (low + high) * 0.5;
        if cubic(middle, x1, x2) < x {
            low = middle;
        } else {
            high = middle;
        }
    }
    (low + high) * 0.5
}

fn cubic(time: f64, first: f64, second: f64) -> f64 {
    let inverse = 1.0 - time;
    3.0 * inverse * inverse * time * first
        + 3.0 * inverse * time * time * second
        + time * time * time
}
