use crate::{visual_scale_valid, Animatable, Vec2};

pub fn max_visual_scale(value: &Animatable<Vec2>) -> Option<(f64, f64)> {
    let values: Vec<Vec2> = match value {
        Animatable::Constant { value } => vec![*value],
        Animatable::Keyframes { keyframes } => {
            let mut values: Vec<Vec2> = keyframes.iter().map(|frame| frame.value).collect();
            for window in keyframes.windows(2) {
                let Some(extrema) = window[0].interpolation.spring_extrema() else {
                    continue;
                };
                for progress in extrema {
                    values.push(Vec2 {
                        x: window[0].value.x + (window[1].value.x - window[0].value.x) * progress,
                        y: window[0].value.y + (window[1].value.y - window[0].value.y) * progress,
                    });
                }
            }
            values
        }
    };
    values
        .into_iter()
        .try_fold((0.0_f64, 0.0_f64), |(x, y), value| {
            visual_scale_valid(value).then_some((x.max(value.x), y.max(value.y)))
        })
}
