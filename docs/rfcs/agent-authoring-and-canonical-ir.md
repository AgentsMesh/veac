# RFC: Agent Authoring and Canonical IR

Status: Accepted

## Decision

VEAC has one agent-oriented authoring language and one canonical JSON execution IR. Historical source compatibility is explicitly out of scope.

```text
.veac -> typed AST -> canonical JSON -> plan -> typed backend bundle -> artifacts
```

## Motivation

A flat attribute language makes every declaration look like a JSON object with different punctuation. Agents cannot reliably distinguish ownership, stable identity, typed references, source/record time, or legal combinations. Open maps also push diagnostics into FFmpeg, where source spans and domain intent are gone.

## Authoring Rules

1. Declaration heads contain a stable entity kind and ID only.
2. Ownership is expressed by blocks.
3. Cross-owner behavior is a first-class relation.
4. Value variants and units are closed and typed.
5. Repeated dynamic values use `Parameter<T> = constant | curve`.
6. Unknown declarations, fields, enum values, and units fail.
7. Duplicate singleton fields fail.
8. References resolve by expected type.
9. The AST retains spans and omitted state.
10. Lowering is explicit and followed by canonical validation.

## Core Model

```text
Project   = settings + resources + entry + multicams + sequences + annotations + outputs
Sequence  = layers + relations + applies
Layer     = ordered items + optional bus route
Item      = source + record + optional mapping + modifiers + optional template slot
```

Source, modifier, relation, annotation, generator, audio processor, color stage, text layout, and output are closed variants.

## Canonical Boundary

Canonical IR uses exact rational time, stable typed IDs, deterministic ordering, and serde schemas. It is the boundary for editing, template filling, planning, caching, signatures, and external tooling.

The following accepted media fragment is kept executable by the IR documentation test:

```json,veac-media
{
  "id": "med_logo",
  "identity": null,
  "kind": "image",
  "metadata": {},
  "probe": null,
  "source": {
    "type": "file",
    "uri": "assets/logo.png"
  },
  "stream_intent": {
    "audio": { "type": "disabled" },
    "video": { "type": "auto" }
  }
}
```

The authoring AST does not reuse canonical structs. Surface values may have richer provenance and omission information; lowering maps them to the minimal executable canonical fact.

## Time

Record time, item-local time, source time, and output frame/sample time are distinct domains. Authoring literals lower exactly to the project timescale. Parameters use item-local time. Source mappings map item-local record duration into source time. Multicam switches partition item-local time.

## Relations

Transition, matte, sidechain, group, and AV-link are relation facts. Planner projections are derived, never independently authored. A canonical validator must reject divergence between a relation and any persisted execution projection.

## Templates

Template slots are item-owned constraints; the item ID is slot identity. Bindings are separate versioned request artifacts. Fill proposals produce atomic edit batches and validate the candidate project before commit.

## Outputs

Each authoring output owns one closed encoding variant. Lowering creates stable render config and deliverable IDs. Output compatibility, filenames, image patterns, caption track selection, and audio source kinds are validated before execution.

## Consequences

- The old frontend is deleted, not maintained behind a compatibility flag.
- Examples and docs compile against the same frontend as the CLI.
- New mechanisms require parse, format, lower, canonical, plan/codegen, and proportional E2E coverage.
- Planner/backend limitations may withhold a surface primitive rather than accept syntax that cannot execute faithfully.
- All controlled implementation and tooling files remain below 200 lines.
