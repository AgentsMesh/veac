# Timeline and Mapping

Sequences own typed layers; layers own items. Layer kinds are `video`, `visual`, `audio`, and `caption`.

```veac
sequence main {
  layer video picture {
    item opening {
      source media resource interview;
      record { at 4s; duration 6s; }
      mapping linear { from 12s; to 18s; }
      modifiers { composite visible { opacity 100%; z-index 0; blend normal; } }
    }
  }
}
```

Three time domains stay distinct:

- `record`: placement in the owning sequence.
- item local time: `[0, record.duration]`, used by parameters and modifier ranges.
- source time: positions in media, multicam angles, or another source domain.

Media source mapping is one of:

```veac
mapping linear { from 2s; to 6s; outside hold-both; }

mapping curve {
  key start { at 0s; source 2s; interpolation linear; }
  key end { at 4s; source 8s; interpolation linear; }
}

mapping freeze { source 3.5s; }
```

Decreasing linear bounds express reverse playback. `outside` is one of
`strict`, `hold-first`, `hold-last`, or `hold-both`; looping is never implicit.

`Parameter<T>` uses the same constant-or-curve shape for position, scale, rotation, opacity, mask
geometry, numeric effect parameters, and text animation channels. Boolean and color effect
parameters are static typed values; they do not accept curves.

A key's `interpolation` controls its outgoing segment to the next key. Use `hold` on the terminal
key to state that its final value remains fixed; terminal interpolation never reshapes the segment
that enters it.

Nested sequences use `source sequence sequence <id>;`. Their base recursive execution is supported, including cycle and depth validation. Source-time mapping for nested sequence composites is intentionally not yet exposed until planner mapping has the same fidelity as media mapping.

Template slots belong to their item; the item ID is the slot identity:

```veac
template-slot media {
  accepts video-or-image;
  fill fit-duration;
  label "Hero media";
}

template-slot text;
```

Runtime fill requests are separate artifacts, not timeline declarations.
