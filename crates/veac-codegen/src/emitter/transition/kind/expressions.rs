use veac_plan::canonical::{CardinalDirection, CircleDirection, TransitionKind, ZoomDirection};

const PROGRESS: &str = "1-P";

pub(super) fn custom(value: &TransitionKind) -> String {
    match value {
        TransitionKind::Wipe {
            direction,
            angle_degrees,
            softness,
        } => wipe(*direction, *angle_degrees, *softness),
        TransitionKind::Slide { direction, amount } => slide(*direction, *amount),
        TransitionKind::Zoom { direction, amount } => zoom(*direction, *amount),
        TransitionKind::Circle {
            direction,
            softness,
        } => circle(*direction, *softness),
        TransitionKind::Pixelize { amount } => pixelize(*amount),
        TransitionKind::Dissolve | TransitionKind::Fade { .. } => {
            unreachable!("native transitions do not use custom expressions")
        }
    }
}

fn wipe(direction: CardinalDirection, angle: f64, softness: f64) -> String {
    let base = match direction {
        CardinalDirection::Left => 180.0,
        CardinalDirection::Right => 0.0,
        CardinalDirection::Up => -90.0,
        CardinalDirection::Down => 90.0,
    };
    let radians = format!("{}*PI/180", super::super::super::time::number(base + angle));
    let projection = format!(
        "0.5+((X/W-0.5)*cos({radians})+(Y/H-0.5)*sin({radians}))/(abs(cos({radians}))+abs(sin({radians})))"
    );
    let softness = super::super::super::time::number(softness);
    let weight = format!(
        "clip((({PROGRESS})-({projection})+({softness})/2)/max({softness}\\,0.000001)\\,0\\,1)"
    );
    format!("A*(1-({weight}))+B*({weight})")
}

fn slide(direction: CardinalDirection, amount: f64) -> String {
    let q = format!(
        "pow({PROGRESS}\\,{})",
        super::super::super::time::number(amount)
    );
    match direction {
        CardinalDirection::Left => choose(
            &format!("gte(X\\,W*(1-{q}))"),
            sample('b', &format!("X-W*(1-{q})"), "Y"),
            sample('a', &format!("X+W*{q}"), "Y"),
        ),
        CardinalDirection::Right => choose(
            &format!("lt(X\\,W*{q})"),
            sample('b', &format!("X+W*(1-{q})"), "Y"),
            sample('a', &format!("X-W*{q}"), "Y"),
        ),
        CardinalDirection::Up => choose(
            &format!("gte(Y\\,H*(1-{q}))"),
            sample('b', "X", &format!("Y-H*(1-{q})")),
            sample('a', "X", &format!("Y+H*{q}")),
        ),
        CardinalDirection::Down => choose(
            &format!("lt(Y\\,H*{q})"),
            sample('b', "X", &format!("Y+H*(1-{q})")),
            sample('a', "X", &format!("Y-H*{q}")),
        ),
    }
}

fn zoom(direction: ZoomDirection, amount: f64) -> String {
    let q = format!(
        "pow({PROGRESS}\\,{})",
        super::super::super::time::number(amount)
    );
    let scale = match direction {
        ZoomDirection::In => q.clone(),
        ZoomDirection::Out => format!("1-({q})"),
    };
    let x = format!("(X-W/2)/max({scale}\\,0.001)+W/2");
    let y = format!("(Y-H/2)/max({scale}\\,0.001)+H/2");
    match direction {
        ZoomDirection::In => format!(
            "if(between({x}\\,0\\,W-1)*between({y}\\,0\\,H-1)\\,A*P+({})*{q}\\,A)",
            sample('b', &x, &y)
        ),
        ZoomDirection::Out => format!(
            "if(between({x}\\,0\\,W-1)*between({y}\\,0\\,H-1)\\,({})*P+B*{q}\\,B)",
            sample('a', &x, &y)
        ),
    }
}

fn circle(direction: CircleDirection, softness: f64) -> String {
    let softness = super::super::super::time::number(softness);
    let radius = "hypot(X-W/2\\,Y-H/2)/hypot(W/2\\,H/2)";
    let edge = match direction {
        CircleDirection::Open => format!("({PROGRESS})-({radius})"),
        CircleDirection::Close => format!("({radius})-P"),
    };
    let weight = format!("clip(0.5+({edge})/max({softness}\\,0.000001)\\,0\\,1)");
    format!("A*(1-({weight}))+B*({weight})")
}

fn pixelize(amount: f64) -> String {
    let amount = super::super::super::time::number(amount);
    let block = format!("1+({amount})*min(W\\,H)*4*P*({PROGRESS})");
    let x = format!("floor(X/({block}))*({block})");
    let y = format!("floor(Y/({block}))*({block})");
    format!(
        "({})*P+({})*({PROGRESS})",
        sample('a', &x, &y),
        sample('b', &x, &y)
    )
}

fn choose(condition: &str, yes: String, no: String) -> String {
    format!("if({condition}\\,{yes}\\,{no})")
}

fn sample(input: char, x: &str, y: &str) -> String {
    format!(
        "if(eq(PLANE\\,0)\\,{input}0({x}\\,{y})\\,if(eq(PLANE\\,1)\\,{input}1({x}\\,{y})\\,if(eq(PLANE\\,2)\\,{input}2({x}\\,{y})\\,{input}3({x}\\,{y}))))"
    )
}
