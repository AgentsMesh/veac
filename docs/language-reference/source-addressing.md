# Stable Source Addressing

`source-index` and `SourceEditBatch` identify authoring nodes through closed, module-qualified semantic
paths. Byte ranges are observations for display and precondition checks; they are never public identity.

## Names

Every name inside a target path or named expression site follows the declaration-name contract: 1 to 128
ASCII bytes matching `[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*`, excluding reserved `true` and `false`.
Leading underscores and kebab-case are valid; dots, Unicode, leading digits, trailing dashes, and repeated
dashes are not. Qualified module identity stays in the separate `module` field. Declarations and expression
symbol segments use the same rule, so every compiled declaration remains directly addressable.

## Project Paths

`target.path` is hierarchy-aware. Declaration paths remain compact:

```text
constant:           constant
component:          component
component_instance: instance
resource:           resource
```

Timeline paths include every owning scope needed to make local IDs unambiguous:

```text
project:  project
sequence: project + sequence
layer:    project + sequence + layer
item:     project + sequence + layer + item
modifier: project + sequence + layer + item + modifier
apply:    project + sequence + apply
stage:    project + sequence + apply + stage
```

Two items may both contain a modifier named `blur`. Their paths remain distinct because item, layer,
sequence, and project identities are part of the target. Agents construct these semantic paths and never
infer identity from a globally flattened leaf ID.

## Definition Paths

Definition paths retain their authoring owner rather than an expanded runtime ID:

```text
component_local_instance: component + instance
component_layer:          component + layer
component_item:           component + layer + item
component_modifier:       component + layer + item + modifier
component_apply:          component + apply
component_stage:          component + apply + stage
preset:                   preset_kind + preset
```

Component-local source spells declarations as `@name`; inventory targets contain canonical `name` without
`@`, while their ranges still select the original `@name` bytes. They never contain `veac-h-*` expansion IDs.
Preset child variants are closed by owner kind: modifier-stack modifiers, effect-pipeline stages, audio
processors, EQ bands, and delivery artifacts cannot alias each other. Text style/layout, color, audio, and
delivery fields use public enums rather than token strings.

## Audio Children

Audio processor and EQ band declarations require explicit owner-local IDs:

```veac
processor limiter final-limiter {
  ceiling -1db;
  attack 1ms;
  release 50ms;
}

processor eq tone-shaper {
  band presence {
    frequency 1000hz;
    gain 1db;
    q 1;
  }
}
```

Their source identities are `preset + processor` and `preset + processor + band`. Processor kind is a closed
field used for validation, not identity. Multiple processors of the same kind are legal when their IDs differ;
duplicate processor IDs and duplicate band IDs are compile errors. Each EQ band exposes independently editable
frequency, gain, and Q sites. Color pipeline sections such as `basic` are semantic singletons, so duplicate
sections are also compile errors. No occurrence index, byte offset, kind, or generated ID becomes identity.
