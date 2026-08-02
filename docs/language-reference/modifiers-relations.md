# Modifiers and Relations

Modifiers are ordered item-local pipelines:

```text
layout | transform | composite | surface | mask | audio | color | effect
```

Layout separates placement from frame fit. Transform owns position, scale, rotation, anchor, crop, and flips. Composite owns opacity, z-index, and blend mode. Mask is a closed shape variant with animatable position, scale, rotation, feather, and expansion.

Surface owns presentation applied to the composed item boundary. It lowers
directly to `VisualProperties.card`; it is not a generated-shape property bag:

```veac
surface raised {
  corner-radius 36px;
  shadow {
    color #000000ff;
    opacity 38%;
    blur 28px;
    offset { x 0px; y 16px; }
  }
}
```

The corner radius is required. A shadow is optional, but when present its
color, opacity, blur, and two-dimensional offset are complete typed values.

Color is an explicit ordered pipeline:

```text
input-space -> working-space -> basic/matrix/hsl/curves/wheels/lut -> output-space
```

Effects have a typed effect kind and a closed parameter union: number, number curve, boolean, or
color. There is no generic text, integer, time, or vector escape hatch. The canonical schema,
authoring parser/lowerer, registry modes, and executable backend catalog are guarded as exact sets;
unknown effect kinds or parameter fields fail during authoring or canonical validation.

Cross-item semantics are first-class relations, not duplicated clip properties:

```text
transition | matte | sidechain | group | av-link
```

```veac
relation transition dissolve-cut {
  endpoints { from item first; to item second; }
  timing { duration 400ms; alignment centered; }
  style { dissolve; }
}

relation group edit-unit {
  members { item picture; item sound; }
}
```

`apply` is a first-class composition primitive. It combines one closed target,
a record range, ordered color/effect stages, and an output mix:

```veac
apply global-grade {
  scope composite-band {
    from layer picture;
    through layer titles;
  }
  record { at 0s; duration 8s; }
  pipeline {
    stage effect contrast {
      type video.color_adjust;
      parameter contrast 1.2;
    }
  }
  mix { opacity 90%; blend soft-light; }
}
```

The target is exactly CompositeBand, Layer, or ItemSet. ItemSet syntax accepts
items and groups, expanding groups to exact sorted member IDs. Apply lowers to
`Sequence.applies`; it never creates adjustment tracks or clips. Pipeline order
is semantic, and mix masks limit the processed result without widening targets.
