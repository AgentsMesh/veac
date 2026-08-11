use super::Interpolation;

const EPSILON: f64 = 1e-12;

impl Interpolation {
    pub fn intermediate_extrema(&self) -> Option<Vec<f64>> {
        match self {
            Self::Spring { .. } => self.spring_extrema(),
            Self::CubicBezier { y1, y2, .. } => cubic_extrema(*y1, *y2),
            _ => Some(Vec::new()),
        }
    }
}

fn cubic_extrema(y1: f64, y2: f64) -> Option<Vec<f64>> {
    if !y1.is_finite() || !y2.is_finite() {
        return None;
    }
    let a = 3.0 * y1 - 3.0 * y2 + 1.0;
    let b = 2.0 * (y2 - 2.0 * y1);
    let c = y1;
    if !a.is_finite() || !b.is_finite() {
        return None;
    }
    let mut roots = if a.abs() <= EPSILON {
        if b.abs() > EPSILON {
            vec![-c / b]
        } else {
            Vec::new()
        }
    } else {
        let discriminant = b * b - 4.0 * a * c;
        if !discriminant.is_finite() {
            return None;
        }
        if discriminant < -EPSILON {
            Vec::new()
        } else {
            let root = discriminant.max(0.0).sqrt();
            vec![(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)]
        }
    };
    if roots.iter().any(|root| !root.is_finite()) {
        return None;
    }
    roots.sort_by(f64::total_cmp);
    let values = roots
        .into_iter()
        .filter(|time| (0.0..1.0).contains(time))
        .map(|time| cubic(time, y1, y2))
        .collect::<Vec<_>>();
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(values)
}

fn cubic(time: f64, first: f64, second: f64) -> f64 {
    let inverse = 1.0 - time;
    3.0 * inverse * inverse * time * first
        + 3.0 * inverse * time * time * second
        + time * time * time
}
