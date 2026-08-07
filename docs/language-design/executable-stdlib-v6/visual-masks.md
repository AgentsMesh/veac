# Visual Composition And Masks

## DomainTypes

```text
VisualStyle, VisualLayout, VisualSurface, Placement, Anchor, FrameChoice, FitMode
Transform2D, TransformMotion, TransformGeometry, Flip, CropChoice
Compositing, BlendMode, CardChoice, ShadowChoice
ColorPipelineChoice
Mask, MaskShape, MaskMotion, MaskEdge
```

The family is split into layout, surface, and mask aggregates. This is a semantic composition model,
not a list of canonical fields and not a fluent setter API.

## Placement And Frame

```veac
anchor_center() -> Anchor
anchor_top_left() -> Anchor
anchor_top() -> Anchor
anchor_top_right() -> Anchor
anchor_left() -> Anchor
anchor_right() -> Anchor
anchor_bottom_left() -> Anchor
anchor_bottom() -> Anchor
anchor_bottom_right() -> Anchor
placement_anchor(anchor: Anchor, inset: Vector) -> Placement
placement_absolute(position: Point) -> Placement
fit_fill() -> FitMode
fit_contain() -> FitMode
fit_cover() -> FitMode
frame_none() -> FrameChoice
frame_sized(width: length, height: length, fit: FitMode) -> FrameChoice
```

Frame dimensions are positive. Anchor inset is canvas-relative; absolute Point keeps explicit length
units. Fit has no string escape hatch.

## Transform And Layout

```veac
flip_none() -> Flip
flip_horizontal() -> Flip
flip_vertical() -> Flip
flip_both() -> Flip
crop_none() -> CropChoice
crop_animated(viewport: RectAnimation) -> CropChoice
transform_motion(position: PointAnimation, scale: VectorAnimation,
                 rotation: AngleAnimation) -> TransformMotion
transform_geometry(shear: Vector, flip: Flip, anchor: Vector,
                   crop: CropChoice) -> TransformGeometry
transform_2d(motion: TransformMotion, geometry: TransformGeometry) -> Transform2D
visual_layout(placement: Placement, frame: FrameChoice,
              transform: Transform2D) -> VisualLayout
```

Scale values are positive, shear components are in `[-2, 2]`, and anchor/crop coordinates are
normalized. Animated crop preserves the first keyframe output extent, matching canonical semantics.

## Compositing And Surface

```veac
blend_normal() -> BlendMode
blend_multiply() -> BlendMode
blend_screen() -> BlendMode
blend_overlay() -> BlendMode
blend_darken() -> BlendMode
blend_lighten() -> BlendMode
blend_color_dodge() -> BlendMode
blend_color_burn() -> BlendMode
blend_hard_light() -> BlendMode
blend_soft_light() -> BlendMode
blend_difference() -> BlendMode
blend_exclusion() -> BlendMode
compositing(z_index: int, blend: BlendMode) -> Compositing
shadow_none() -> ShadowChoice
shadow_present(blur: length, opacity: percent, offset: Vector,
               value: color) -> ShadowChoice
card_none() -> CardChoice
card_present(corner_radius: length, shadow: ShadowChoice) -> CardChoice
visual_surface(opacity: PercentAnimation, compositing: Compositing,
               card: CardChoice) -> VisualSurface
```

Blur and corner radius are non-negative. Surface opacity is typed as normalized percent rather than a
generic scalar. Card and shadow are closed choices, so absence is explicit and immutable.

## Mask Geometry

```veac
mask_linear() -> MaskShape
mask_mirror() -> MaskShape
mask_circle() -> MaskShape
mask_rectangle() -> MaskShape
mask_rounded_rectangle(radius: scalar) -> MaskShape
mask_ellipse() -> MaskShape
mask_polygon(points: list<Vector>) -> MaskShape
mask_heart() -> MaskShape
mask_star() -> MaskShape
mask_path(points: list<Vector>) -> MaskShape
mask_motion(position: VectorAnimation, scale: VectorAnimation,
            rotation: AngleAnimation) -> MaskMotion
mask_edge(feather: LengthAnimation, expansion: LengthAnimation) -> MaskEdge
mask(shape: MaskShape, motion: MaskMotion, edge: MaskEdge, invert: bool) -> Mask
```

Polygon and path cardinality rules match generator geometry. Rounded radius, feather, and scale are
non-negative; expansion may be negative. A Mask is the atomic reusable value and has no per-field
replacement methods.

## Complete Visual Style

```veac
color_pipeline_none() -> ColorPipelineChoice
color_pipeline_present(value: ColorPipeline) -> ColorPipelineChoice
visual_style(layout: VisualLayout, surface: VisualSurface, masks: list<Mask>,
             color: ColorPipelineChoice) -> VisualStyle
Item.with_visual(style: VisualStyle) -> Item
```

`Item.with_visual` is `GraphEmit`, accepts one complete style, and replaces no unrelated Item state.
Mask order is semantic. The color pipeline is evaluated after source sampling and before surface
compositing; attachment fixes that order independently of backend implementation.
