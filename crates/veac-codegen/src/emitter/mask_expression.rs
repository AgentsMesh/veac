use veac_plan::canonical::{Mask, MaskShape, Vec2};

use super::{animation, mask_path, time};

pub(super) fn alpha(mask: &Mask) -> String {
    let point = Coordinates::new(mask);
    let distance = signed_distance(&mask.shape, &point);
    let feather = animation::number(&mask.feather_pixels, "T");
    let expansion = animation::number(&mask.expansion_pixels, "T");
    let coverage =
        format!("clip(0.5+(({distance})+({expansion}))/max(2*({feather})\\,0.000001)\\,0\\,1)");
    let coverage = if mask.invert {
        format!("1-({coverage})")
    } else {
        coverage
    };
    format!("alpha(X\\,Y)*({coverage})")
}

struct Coordinates {
    x: String,
    y: String,
    width: String,
    height: String,
}

impl Coordinates {
    fn new(mask: &Mask) -> Self {
        let px = animation::vec_x(&mask.position, "T");
        let py = animation::vec_y(&mask.position, "T");
        let sx = animation::vec_x(&mask.scale, "T");
        let sy = animation::vec_y(&mask.scale, "T");
        let angle = animation::number(&mask.rotation_degrees, "T");
        let radians = format!("(({angle})*PI/180)");
        let dx = format!("(X-W*({px}))");
        let dy = format!("(Y-H*({py}))");
        Self {
            x: format!("(cos({radians})*{dx}+sin({radians})*{dy})"),
            y: format!("(-sin({radians})*{dx}+cos({radians})*{dy})"),
            width: format!("W*({sx})"),
            height: format!("H*({sy})"),
        }
    }

    fn half_width(&self) -> String {
        format!("({})/2", self.width)
    }

    fn half_height(&self) -> String {
        format!("({})/2", self.height)
    }

    fn short_extent(&self) -> String {
        format!("min({}\\,{})", self.width, self.height)
    }
}

fn signed_distance(shape: &MaskShape, point: &Coordinates) -> String {
    match shape {
        MaskShape::Linear => point.x.clone(),
        MaskShape::Mirror => format!("({})-abs({})", point.half_width(), point.x),
        MaskShape::Circle => circle(point),
        MaskShape::Rectangle => box_distance(point, "0"),
        MaskShape::RoundedRectangle { radius } => {
            let radius = format!("{}*({})", time::number(*radius), point.short_extent());
            box_distance(point, &radius)
        }
        MaskShape::Ellipse => ellipse(point),
        MaskShape::Polygon { points } | MaskShape::Path { points } => path(points, point),
        MaskShape::Heart => heart(point),
        MaskShape::Star => star(point),
    }
}

fn circle(point: &Coordinates) -> String {
    format!(
        "({})/2-hypot({}\\,{})",
        point.short_extent(),
        point.x,
        point.y
    )
}

fn box_distance(point: &Coordinates, radius: &str) -> String {
    let qx = format!("abs({})-(({})-({radius}))", point.x, point.half_width());
    let qy = format!("abs({})-(({})-({radius}))", point.y, point.half_height());
    format!("({radius})-(hypot(max({qx}\\,0)\\,max({qy}\\,0))+min(max({qx}\\,{qy})\\,0))")
}

fn ellipse(point: &Coordinates) -> String {
    // Gradient normalization is axis-exact and first-order accurate at the ellipse edge.
    let hx = point.half_width();
    let hy = point.half_height();
    let k0 = format!("hypot(({})/({hx})\\,({})/({hy}))", point.x, point.y);
    let k1 = format!(
        "hypot(({})/(({hx})*({hx}))\\,({})/(({hy})*({hy})))",
        point.x, point.y
    );
    format!("if(eq({k0}\\,0)\\,min({hx}\\,{hy})\\,({k0})*(1-({k0}))/max({k1}\\,0.000001))")
}

fn path(points: &[Vec2], point: &Coordinates) -> String {
    mask_path::signed_distance(points, &point.x, &point.y, &point.width, &point.height)
}

fn heart(point: &Coordinates) -> String {
    // Convert the implicit curve's gradient back through each local pixel axis.
    let x = format!("({})/(0.45*({}))", point.x, point.width);
    let y = format!("-({})/(0.45*({}))", point.y, point.height);
    let sum = format!("pow({x}\\,2)+pow({y}\\,2)-1");
    let implicit = format!("pow({sum}\\,3)-pow({x}\\,2)*pow({y}\\,3)");
    let gx = format!(
        "(6*({x})*pow({sum}\\,2)-2*({x})*pow({y}\\,3))/(0.45*({}))",
        point.width
    );
    let gy = format!(
        "(6*({y})*pow({sum}\\,2)-3*pow({x}\\,2)*pow({y}\\,2))/(0.45*({}))",
        point.height
    );
    format!("-({implicit})/max(hypot({gx}\\,{gy})\\,0.000001)")
}

fn star(point: &Coordinates) -> String {
    // The radial boundary keeps its authored shape; its gradient supplies pixel distance.
    let x = format!("({})/({})", point.x, point.width);
    let y = format!("({})/({})", point.y, point.height);
    let radius = format!("hypot({x}\\,{y})");
    let angle = format!("atan2({y}\\,{x})");
    let wave = format!("sin(5*({angle}))");
    let boundary = format!("0.28+0.12*cos(5*({angle}))");
    let squared = format!("max(pow({radius}\\,2)\\,0.000001)");
    let gx = format!(
        "(0.6*({y})*({wave})/({squared})-({x})/max({radius}\\,0.000001))/({})",
        point.width
    );
    let gy = format!(
        "(-0.6*({x})*({wave})/({squared})-({y})/max({radius}\\,0.000001))/({})",
        point.height
    );
    format!(
        "if(eq({radius}\\,0)\\,0.16*({})\\,({boundary}-({radius}))/max(hypot({gx}\\,{gy})\\,0.000001))",
        point.short_extent()
    )
}

#[cfg(test)]
mod tests;
