use super::{Interpolation, SpringCoefficients};

const SPRING_EPSILON: f64 = 1e-9;
const MAX_SPRING_FREQUENCY: f64 = 16.0;
const MAX_SPRING_DECAY: f64 = 64.0;
const MAX_SPRING_VELOCITY: f64 = 128.0;

impl Interpolation {
    pub fn evaluate(&self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Hold => 0.0,
            Self::Linear => progress,
            Self::EaseIn => progress * progress,
            Self::EaseOut => 1.0 - (1.0 - progress).powi(2),
            Self::EaseInOut => progress * progress * (3.0 - 2.0 * progress),
            Self::Spring { .. } => self.spring_value(progress).unwrap_or(f64::NAN),
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
            Self::Hold | Self::Spring { .. } => None,
        }
    }

    pub(crate) fn parameter_for_x(&self, x: f64) -> Option<f64> {
        let controls = self.cubic_controls()?;
        Some(parameter_for_x(x, controls[0], controls[2]))
    }

    pub fn spring_coefficients(&self) -> Option<SpringCoefficients> {
        let Self::Spring {
            frequency,
            decay,
            initial_velocity,
        } = self
        else {
            return None;
        };
        if !frequency.is_finite()
            || !decay.is_finite()
            || !initial_velocity.is_finite()
            || *frequency <= 0.0
            || *frequency > MAX_SPRING_FREQUENCY
            || *decay <= 0.0
            || *decay > MAX_SPRING_DECAY
            || initial_velocity.abs() > MAX_SPRING_VELOCITY
        {
            return None;
        }
        let angular_frequency = 2.0 * std::f64::consts::PI * frequency;
        let attenuation = (-decay).exp();
        let phase_sine = angular_frequency.sin();
        let response =
            1.0 - attenuation * (angular_frequency.cos() + decay * phase_sine / angular_frequency);
        if !response.is_finite() || response.abs() <= SPRING_EPSILON {
            return None;
        }
        let equilibrium =
            (1.0 - initial_velocity * attenuation * phase_sine / angular_frequency) / response;
        let sine = (initial_velocity - decay * equilibrium) / angular_frequency;
        let derivative_sine = equilibrium * angular_frequency - decay * sine;
        if !equilibrium.is_finite() || !sine.is_finite() || !derivative_sine.is_finite() {
            return None;
        }
        let coefficients = SpringCoefficients {
            angular_frequency,
            equilibrium,
            sine,
        };
        let start = spring_response(coefficients, *decay, 0.0);
        let end = spring_response(coefficients, *decay, 1.0);
        if !start.is_finite()
            || !end.is_finite()
            || start.abs() > SPRING_EPSILON
            || (end - 1.0).abs() > SPRING_EPSILON
        {
            return None;
        }
        Some(coefficients)
    }

    pub fn spring_value(&self, progress: f64) -> Option<f64> {
        let coefficients = self.spring_coefficients()?;
        let Self::Spring { decay, .. } = self else {
            return None;
        };
        Some(spring_response(coefficients, *decay, progress))
    }

    pub fn spring_derivative(&self, progress: f64) -> Option<f64> {
        let coefficients = self.spring_coefficients()?;
        let Self::Spring {
            decay,
            initial_velocity,
            ..
        } = self
        else {
            return None;
        };
        let phase = coefficients.angular_frequency * progress;
        let sine =
            coefficients.equilibrium * coefficients.angular_frequency - decay * coefficients.sine;
        Some((-decay * progress).exp() * (initial_velocity * phase.cos() + sine * phase.sin()))
    }

    pub fn spring_extrema(&self) -> Option<Vec<f64>> {
        let coefficients = self.spring_coefficients()?;
        let Self::Spring {
            decay,
            initial_velocity,
            ..
        } = self
        else {
            return None;
        };
        let sine =
            coefficients.equilibrium * coefficients.angular_frequency - decay * coefficients.sine;
        let mut phase = (-initial_velocity)
            .atan2(sine)
            .rem_euclid(std::f64::consts::PI);
        if phase <= SPRING_EPSILON {
            phase += std::f64::consts::PI;
        }
        let mut values = Vec::new();
        while phase < coefficients.angular_frequency {
            values.push(self.spring_value(phase / coefficients.angular_frequency)?);
            phase += std::f64::consts::PI;
        }
        Some(values)
    }
}

fn spring_response(coefficients: SpringCoefficients, decay: f64, progress: f64) -> f64 {
    let phase = coefficients.angular_frequency * progress;
    coefficients.equilibrium
        + (-decay * progress).exp()
            * (-coefficients.equilibrium * phase.cos() + coefficients.sine * phase.sin())
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
