use veac_plan::canonical::{
    Animatable, Interpolation, Keyframe, Length, LengthUnit, Point, Rect, Vec2,
};

use super::time;

pub(crate) fn number(value: &Animatable<f64>, clock: &str) -> String {
    expression(value, clock, |value| time::number(*value))
}

pub(crate) fn vec_x(value: &Animatable<Vec2>, clock: &str) -> String {
    expression(value, clock, |value| time::number(value.x))
}

pub(crate) fn vec_y(value: &Animatable<Vec2>, clock: &str) -> String {
    expression(value, clock, |value| time::number(value.y))
}

pub(crate) fn point_x(value: &Animatable<Point>, clock: &str, extent: &str) -> String {
    expression(value, clock, |value| length(value.x, extent))
}

pub(crate) fn point_y(value: &Animatable<Point>, clock: &str, extent: &str) -> String {
    expression(value, clock, |value| length(value.y, extent))
}

pub(crate) fn rect_x(value: &Animatable<Rect>, clock: &str) -> String {
    expression(value, clock, |value| time::number(value.x))
}

pub(crate) fn rect_y(value: &Animatable<Rect>, clock: &str) -> String {
    expression(value, clock, |value| time::number(value.y))
}

pub(crate) fn rect_width(value: &Animatable<Rect>, clock: &str) -> String {
    expression(value, clock, |value| time::number(value.width))
}

pub(crate) fn rect_height(value: &Animatable<Rect>, clock: &str) -> String {
    expression(value, clock, |value| time::number(value.height))
}

fn expression<T>(value: &Animatable<T>, clock: &str, render: impl Fn(&T) -> String) -> String {
    match value {
        Animatable::Constant { value } => render(value),
        Animatable::Keyframes { keyframes } => curve(keyframes, clock, render),
    }
}

fn curve<T>(keyframes: &[Keyframe<T>], clock: &str, render: impl Fn(&T) -> String) -> String {
    let Some(first) = keyframes.first() else {
        return "0".to_owned();
    };
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

fn easing(progress: &str, interpolation: &Interpolation) -> String {
    match interpolation {
        Interpolation::Hold => "0".to_owned(),
        Interpolation::Linear => progress.to_owned(),
        Interpolation::EaseIn => format!("pow({progress}\\,2)"),
        Interpolation::EaseOut => format!("1-pow(1-({progress})\\,2)"),
        Interpolation::EaseInOut => {
            format!("pow({progress}\\,2)*(3-2*({progress}))")
        }
        Interpolation::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(progress, *x1, *y1, *x2, *y2),
    }
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

fn length(value: Length, extent: &str) -> String {
    match value.unit {
        LengthUnit::Pixels => time::number(value.value),
        LengthUnit::Normalized => format!("{extent}*{}", time::number(value.value)),
        LengthUnit::Percent => format!("{extent}*{}/100", time::number(value.value)),
    }
}
