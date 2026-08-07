use veac_plan::canonical::{Mask, MaskShape, Vec2};
use veac_plan::ResolvedRenderPlan;

use super::{animation, mask_path, process_owner::ProcessOwner, time};

pub(super) fn alpha(plan: &ResolvedRenderPlan, owner: ProcessOwner<'_>, mask: &Mask) -> String {
    let point = Coordinates::new(plan, owner, mask);
    let distance = signed_distance(&mask.shape, &point);
    let feather = animation::number(plan, owner, &mask.feather_pixels, "T");
    let expansion = animation::number(plan, owner, &mask.expansion_pixels, "T");
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
    fn new(plan: &ResolvedRenderPlan, owner: ProcessOwner<'_>, mask: &Mask) -> Self {
        let px = animation::vec_x(plan, owner, &mask.position, "T");
        let py = animation::vec_y(plan, owner, &mask.position, "T");
        let sx = animation::vec_x(plan, owner, &mask.scale, "T");
        let sy = animation::vec_y(plan, owner, &mask.scale, "T");
        let angle = animation::number(plan, owner, &mask.rotation_degrees, "T");
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
    path(&star_points(), point)
}

fn star_points() -> [Vec2; 10] {
    // Outer radius 0.45 alternates with the pentagram intersection radius 0.45 / phi^2.
    [
        Vec2 { x: 0.5, y: 0.05 },
        Vec2 {
            x: 0.601031294730407,
            y: 0.360942352531274,
        },
        Vec2 {
            x: 0.927975432332819,
            y: 0.360942352531274,
        },
        Vec2 {
            x: 0.663472068801206,
            y: 0.553115294937453,
        },
        Vec2 {
            x: 0.764503363531613,
            y: 0.864057647468726,
        },
        Vec2 {
            x: 0.5,
            y: 0.671884705062547,
        },
        Vec2 {
            x: 0.235496636468387,
            y: 0.864057647468726,
        },
        Vec2 {
            x: 0.336527931198794,
            y: 0.553115294937453,
        },
        Vec2 {
            x: 0.072024567667181,
            y: 0.360942352531274,
        },
        Vec2 {
            x: 0.398968705269593,
            y: 0.360942352531274,
        },
    ]
}

#[cfg(test)]
mod tests;
