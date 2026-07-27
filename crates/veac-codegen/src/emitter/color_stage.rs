use veac_plan::canonical::{BasicColorAdjustment, HslAdjustment, HueRange, LiftGammaGain};
use veac_plan::ResolvedColorStage;

use super::{color_lut, process_owner::ProcessOwner, time, CodegenErrors, EmitContext};

mod curves;
mod matrix;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    input: String,
    stage: &ResolvedColorStage,
) -> Result<String, CodegenErrors> {
    match stage {
        ResolvedColorStage::Basic { adjustment } => Ok(basic(context, input, *adjustment)),
        ResolvedColorStage::Matrix { adjustment } => Ok(matrix::apply(context, input, *adjustment)),
        ResolvedColorStage::Hsl { adjustment } => {
            Ok(filter(context, input, hsl(*adjustment), "hslv"))
        }
        ResolvedColorStage::Curves { curves } => Ok(curves::apply(context, input, curves)),
        ResolvedColorStage::Wheels { wheels } => {
            Ok(filter(context, input, wheels_filter(*wheels), "wheelsv"))
        }
        ResolvedColorStage::Lut { application } => {
            color_lut::apply(context, owner, input, application)
        }
    }
}

fn basic(context: &mut EmitContext<'_>, mut input: String, value: BasicColorAdjustment) -> String {
    if value.exposure_stops != 0.0 {
        let factor = number(2.0_f64.powf(value.exposure_stops));
        let channel = format!("clip(val*{factor}\\,0\\,maxval)");
        input = filter(
            context,
            input,
            format!("lutrgb=r='{channel}':g='{channel}':b='{channel}'"),
            "exposurev",
        );
    }
    if value.temperature_kelvin != 6500.0 {
        input = filter(
            context,
            input,
            format!(
                "colortemperature=temperature={}",
                number(value.temperature_kelvin)
            ),
            "temperaturev",
        );
    }
    if value.tint != 0.0 {
        input = filter(
            context,
            input,
            format!("colorbalance=gm={}:pl=true", number(value.tint)),
            "tintv",
        );
    }
    if value.highlights != 0.0 || value.shadows != 0.0 || value.fade != 0.0 {
        let black = value.fade * 0.15;
        let shadow = (0.25 + value.shadows * 0.25).clamp(0.0, 1.0);
        let highlight = (0.75 + value.highlights * 0.25).clamp(0.0, 1.0);
        let white = 1.0 - value.fade * 0.05;
        let points = format!(
            "0/{} 0.25/{} 0.75/{} 1/{}",
            number(black),
            number(shadow),
            number(highlight),
            number(white)
        );
        input = filter(
            context,
            input,
            format!("curves=master='{points}':interp=pchip"),
            "basiccurvev",
        );
    }
    input
}

fn hsl(value: HslAdjustment) -> String {
    let colors = match value.range {
        HueRange::Red => "r",
        HueRange::Yellow => "y",
        HueRange::Green => "g",
        HueRange::Cyan => "c",
        HueRange::Blue => "b",
        HueRange::Magenta => "m",
    };
    format!(
        "huesaturation=hue={}:saturation={}:intensity={}:colors={colors}",
        number(value.hue_degrees),
        number(value.saturation),
        number(value.lightness)
    )
}

fn wheels_filter(value: LiftGammaGain) -> String {
    format!(
        "colorbalance=rs={}:gs={}:bs={}:rm={}:gm={}:bm={}:rh={}:gh={}:bh={}:pl=true",
        number(value.lift.red),
        number(value.lift.green),
        number(value.lift.blue),
        number(value.gamma.red),
        number(value.gamma.green),
        number(value.gamma.blue),
        number(value.gain.red),
        number(value.gain.green),
        number(value.gain.blue)
    )
}

fn filter(context: &mut EmitContext<'_>, input: String, value: String, prefix: &str) -> String {
    context.graph.filter(&[&input], value, prefix)
}

fn number(value: f64) -> String {
    time::number(value)
}
