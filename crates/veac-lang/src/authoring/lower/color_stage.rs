use crate::authoring::{ColorCurvePointDecl, ColorStageDecl, ColorWheelDecl, NumberLiteral};
use veac_ir::{
    BasicColorAdjustment, ColorCurves, ColorStage, ColorWheel, CurvePoint, HslAdjustment,
    LiftGammaGain, LutApplication, RgbMatrixAdjustment, ToneCurve, ToneCurveInterpolation,
};

use super::context::Context;
use super::{color_stage_enum, ids, value};

pub(super) fn lower(ctx: &mut Context, declaration: &ColorStageDecl) -> Option<ColorStage> {
    Some(match declaration {
        ColorStageDecl::Basic(value) => ColorStage::Basic {
            adjustment: BasicColorAdjustment {
                exposure_stops: scalar(ctx, &value.exposure, "stops")?,
                temperature_kelvin: scalar(ctx, &value.temperature, "k")?,
                tint: unitless(ctx, &value.tint)?,
                highlights: unitless(ctx, &value.highlights)?,
                shadows: unitless(ctx, &value.shadows)?,
                fade: value::scale(ctx, &value.fade)?,
            },
        },
        ColorStageDecl::Matrix(value) => ColorStage::Matrix {
            adjustment: RgbMatrixAdjustment {
                matrix: [
                    unitless(ctx, &value.rows[0][0])?,
                    unitless(ctx, &value.rows[0][1])?,
                    unitless(ctx, &value.rows[0][2])?,
                    unitless(ctx, &value.rows[1][0])?,
                    unitless(ctx, &value.rows[1][1])?,
                    unitless(ctx, &value.rows[1][2])?,
                    unitless(ctx, &value.rows[2][0])?,
                    unitless(ctx, &value.rows[2][1])?,
                    unitless(ctx, &value.rows[2][2])?,
                ],
                offset: [
                    unitless(ctx, &value.offset[0])?,
                    unitless(ctx, &value.offset[1])?,
                    unitless(ctx, &value.offset[2])?,
                ],
            },
        },
        ColorStageDecl::Hsl(value) => ColorStage::Hsl {
            adjustment: HslAdjustment {
                range: color_stage_enum::hue_range(ctx, &value.range)?,
                hue_degrees: scalar(ctx, &value.hue, "deg")?,
                saturation: unitless(ctx, &value.saturation)?,
                lightness: unitless(ctx, &value.lightness)?,
            },
        },
        ColorStageDecl::Curves(value) => {
            let interpolation = color_stage_enum::curve_interpolation(ctx, &value.interpolation)?;
            ColorStage::Curves {
                curves: ColorCurves {
                    luma: curve(ctx, value.luma.as_deref(), interpolation)?,
                    red: curve(ctx, value.red.as_deref(), interpolation)?,
                    green: curve(ctx, value.green.as_deref(), interpolation)?,
                    blue: curve(ctx, value.blue.as_deref(), interpolation)?,
                },
            }
        }
        ColorStageDecl::Wheels(value) => ColorStage::Wheels {
            wheels: LiftGammaGain {
                lift: wheel(ctx, &value.lift)?,
                gamma: wheel(ctx, &value.gamma)?,
                gain: wheel(ctx, &value.gain)?,
            },
        },
        ColorStageDecl::Lut(value) => ColorStage::Lut {
            application: LutApplication {
                material_id: ids::material(ctx, &value.resource)?,
                interpolation: color_stage_enum::lut_interpolation(ctx, &value.interpolation)?,
            },
        },
    })
}

fn curve(
    ctx: &mut Context,
    points: Option<&[ColorCurvePointDecl]>,
    interpolation: ToneCurveInterpolation,
) -> Option<Option<ToneCurve>> {
    match points {
        Some(points) => Some(Some(ToneCurve {
            points: points
                .iter()
                .map(|point| {
                    Some(CurvePoint {
                        input: value::scale(ctx, &point.input)?,
                        output: value::scale(ctx, &point.output)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
            interpolation,
        })),
        None => Some(None),
    }
}

fn wheel(ctx: &mut Context, value: &ColorWheelDecl) -> Option<ColorWheel> {
    Some(ColorWheel {
        red: unitless(ctx, &value.red)?,
        green: unitless(ctx, &value.green)?,
        blue: unitless(ctx, &value.blue)?,
    })
}

fn scalar(ctx: &mut Context, value: &NumberLiteral, unit: &str) -> Option<f64> {
    super::value::scalar(ctx, value, unit)
}

fn unitless(ctx: &mut Context, value: &NumberLiteral) -> Option<f64> {
    super::value::unitless(ctx, value)
}
