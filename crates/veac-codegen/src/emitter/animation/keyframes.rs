use veac_plan::canonical::{Interpolation, Keyframe};

use crate::emitter::time;

pub(super) fn curve<T>(
    keyframes: &[Keyframe<T>],
    clock: &str,
    render: impl Fn(&T) -> String,
) -> String {
    let Some(first) = keyframes.first() else {
        return "0".to_owned();
    };
    let first_value = render(&first.value);
    if keyframes
        .iter()
        .skip(1)
        .all(|keyframe| render(&keyframe.value) == first_value)
    {
        return first_value;
    }
    let mut result = render(&keyframes.last().expect("first keyframe exists").value);
    for pair in keyframes.windows(2).rev() {
        let left = &pair[0];
        let right = &pair[1];
        let start = time::seconds(left.time);
        let duration = time::seconds_delta(left.time, right.time);
        let progress = format!("clip(({clock}-{start})/{duration}\\,0\\,1)");
        let eased = easing(&progress, &left.interpolation);
        let value = lerp(&render(&left.value), &render(&right.value), &eased);
        result = format!(
            "if(lt({clock}\\,{})\\,{value}\\,{result})",
            time::seconds(right.time)
        );
    }
    format!(
        "if(lte({clock}\\,{})\\,{}\\,{result})",
        time::seconds(first.time),
        render(&first.value)
    )
}

fn lerp(left: &str, right: &str, progress: &str) -> String {
    format!("({left})+(({right})-({left}))*({progress})")
}

pub(super) fn easing(progress: &str, interpolation: &Interpolation) -> String {
    match interpolation {
        Interpolation::Hold => "0".to_owned(),
        Interpolation::Linear => progress.to_owned(),
        Interpolation::EaseIn => format!("pow({progress}\\,2)"),
        Interpolation::EaseOut => format!("1-pow(1-({progress})\\,2)"),
        Interpolation::EaseInOut => format!("pow({progress}\\,2)*(3-2*({progress}))"),
        Interpolation::Spring { decay, .. } => spring(progress, *decay, interpolation),
        Interpolation::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(progress, *x1, *y1, *x2, *y2),
    }
}

fn spring(progress: &str, decay: f64, interpolation: &Interpolation) -> String {
    let coefficients = interpolation
        .spring_coefficients()
        .expect("validated spring interpolation");
    let decay = spring_number(decay);
    let frequency = spring_number(coefficients.angular_frequency);
    let equilibrium = spring_number(coefficients.equilibrium);
    let sine = spring_number(coefficients.sine);
    let phase = format!("{frequency}*({progress})");
    format!(
        "({equilibrium})+exp(-{decay}*({progress}))*\
         (-({equilibrium})*cos({phase})+({sine})*sin({phase}))"
    )
}

fn spring_number(value: f64) -> String {
    format!("{value:.17e}")
}

fn cubic_bezier(progress: &str, x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let parameter = "ld(0)";
    let x = cubic(parameter, x1, x2);
    let root = format!("root(({x})-({progress})\\,1)");
    format!("st(1\\,{root});{}", cubic("ld(1)", y1, y2))
}

fn cubic(parameter: &str, first: f64, second: f64) -> String {
    let first = time::number(first);
    let second = time::number(second);
    format!(
        "3*(1-({parameter}))*(1-({parameter}))*({parameter})*{first}+\
         3*(1-({parameter}))*({parameter})*({parameter})*{second}+\
         ({parameter})*({parameter})*({parameter})"
    )
}
