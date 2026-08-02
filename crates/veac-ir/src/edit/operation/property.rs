use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "property", content = "value", rename_all = "snake_case")]
pub enum VisualProperty {
    Placement(Placement),
    Frame(Option<Frame>),
    Position(Animatable<Point>),
    Scale(Animatable<Vec2>),
    Shear(Vec2),
    FlipHorizontal(bool),
    FlipVertical(bool),
    RotationDegrees(Animatable<f64>),
    Anchor(Vec2),
    Crop(Option<Animatable<Rect>>),
    Opacity(Animatable<f64>),
    Compositing(Compositing),
    Masks(Vec<Mask>),
    Card(Option<CardStyle>),
    ColorPipeline(Option<ColorPipeline>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "property", content = "value", rename_all = "snake_case")]
pub enum AudioProperty {
    Gain(Animatable<f64>),
    Pan(Animatable<f64>),
    Muted(bool),
    Normalize(bool),
    PitchPolicy(PitchPolicy),
    Processors(Vec<AudioProcessor>),
    Crossfade(Option<AudioCrossfade>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "property", content = "value", rename_all = "snake_case")]
pub enum TextProperty {
    Font(FontRef),
    FallbackFonts(Vec<FontRef>),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    SizePixels(f64),
    Color(Color),
    TrackingPixels(f64),
    LineHeight(f64),
    Layout(TextLayout),
    WritingMode(TextWritingMode),
    Orientation(TextOrientation),
    Path(Option<TextPath>),
    Background(Option<TextBackground>),
    Outline(Option<TextOutline>),
    Shadow(Option<Shadow>),
    Spans(Vec<TextSpan>),
    Animation(Option<TextAnimation>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectParameterEdit {
    Set {
        clip_id: ItemId,
        effect_id: EffectId,
        name: String,
        value: ParameterValue,
    },
    Remove {
        clip_id: ItemId,
        effect_id: EffectId,
        name: String,
    },
}
