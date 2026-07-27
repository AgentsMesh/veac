use veac_plan::canonical::{Mask, MaskShape};

use super::{animation, mask_path};

pub(super) fn alpha(mask: &Mask) -> String {
    let coordinates = Coordinates::new(mask);
    let distance = signed_distance(&mask.shape, &coordinates);
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
    u: String,
    v: String,
    pixels: String,
}

impl Coordinates {
    fn new(mask: &Mask) -> Self {
        let px = animation::vec_x(&mask.position, "T");
        let py = animation::vec_y(&mask.position, "T");
        let sx = animation::vec_x(&mask.scale, "T");
        let sy = animation::vec_y(&mask.scale, "T");
        let angle = animation::number(&mask.rotation_degrees, "T");
        let radians = format!("(({angle})*PI/180)");
        let dx = format!("(X/W-({px}))");
        let dy = format!("(Y/H-({py}))");
        Self {
            u: format!("0.5+(cos({radians})*{dx}+sin({radians})*{dy})/({sx})"),
            v: format!("0.5+(-sin({radians})*{dx}+cos({radians})*{dy})/({sy})"),
            pixels: format!("min(W\\,H)*min({sx}\\,{sy})"),
        }
    }
}

fn signed_distance(shape: &MaskShape, point: &Coordinates) -> String {
    let u = &point.u;
    let v = &point.v;
    let normalized = match shape {
        MaskShape::Linear => format!("({u})-0.5"),
        MaskShape::Mirror => format!("0.5-abs(({u})-0.5)"),
        MaskShape::Circle | MaskShape::Ellipse => {
            format!("0.5-hypot(({u})-0.5\\,({v})-0.5)")
        }
        MaskShape::Rectangle => {
            format!("min(0.5-abs(({u})-0.5)\\,0.5-abs(({v})-0.5))")
        }
        MaskShape::RoundedRectangle { radius } => rounded_rectangle(u, v, *radius),
        MaskShape::Star => {
            format!("0.28+0.12*cos(5*atan2(({v})-0.5\\,({u})-0.5))-hypot(({u})-0.5\\,({v})-0.5)")
        }
        MaskShape::Heart => heart(u, v),
        MaskShape::Polygon { points } | MaskShape::Path { points } => {
            return mask_path::signed_distance(points, u, v, &point.pixels)
        }
    };
    format!("({normalized})*({})", point.pixels)
}

fn rounded_rectangle(u: &str, v: &str, radius: f64) -> String {
    let extent = 0.5 - radius;
    let qx = format!("abs(({u})-0.5)-{extent}");
    let qy = format!("abs(({v})-0.5)-{extent}");
    format!("{radius}-(hypot(max({qx}\\,0)\\,max({qy}\\,0))+min(max({qx}\\,{qy})\\,0))")
}

fn heart(u: &str, v: &str) -> String {
    let x = format!("((({u})-0.5)/0.45)");
    let y = format!("((0.5-({v}))/0.45)");
    let implicit = format!("pow(pow({x}\\,2)+pow({y}\\,2)-1\\,3)-pow({x}\\,2)*pow({y}\\,3)");
    format!("-({implicit})")
}
