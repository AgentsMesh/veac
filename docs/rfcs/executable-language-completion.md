# RFC: Executable VEAC Completion

Status: Implemented

## Decision

VEAC completion means one executable, strongly typed source language owns every supported editing
mechanism. A capability is complete only when executable source constructs it, source editing can
rebuild it, canonical IR validates it, planning preserves it, and a backend can execute it.

The version targets for this clean break are:

```text
Domain opset:        v7 with published Temporal availability and lowering contracts
Core format:         v10 with verified closure parameter stages
Canonical IR schema: v9 with typed effects, captions, executable manifest and temporal programs
Render plan:         v6 with reachable temporal bindings
```

The clean break is complete; no released contract advertises a partially migrated domain. Historical
source compatibility is not a goal.

## Domain Algebra

The standard library is a closed algebra, not a mirror of JSON fields. Each family has immutable typed
description values, explicit variant constructors, and a small owner attachment operation:

| Order | Family | Description values | Owner operation |
| ---: | --- | --- | --- |
| 1 | settings | project, sequence, track state/routing | construct project, sequence, layer |
| 2 | material | locator, identity, stream selection, media kind | attach resource to project |
| 3 | source time | media/sequence source, freeze, mapping/curve | construct item source |
| 4 | generator | silence, gradient, shape, paint, stroke | construct generated source |
| 5 | visual | layout, frame, transform, crop, surface, blend, shadow | attach visual style to item |
| 6 | mask/matte | closed mask geometry and matte relation | attach masks or relation |
| 7 | color | ordered closed color stages and pipeline | attach color pipeline |
| 8 | text | rich spans, layout, path, whole/word animation | construct text/caption item |
| 9 | audio | gain, pan, fades, pitch, processor chain, routing | attach audio style |
| 10 | relation | transition mechanisms, group, AV link, sidechain | attach relation to sequence |
| 11 | apply | target, ordered stages, mask, mix | attach apply to sequence |
| 12 | multicam | angles, switches, group source | attach group to project |
| 13 | template | slot kind, fill mode, constraints | attach item contract |
| 14 | annotation | target, span, closed payload, provenance | attach annotation to project |
| 15 | delivery | raster, codec recipes, artifacts, package | attach delivery to project |
| 16 | effect | typed built-ins, then versioned plugin descriptors | attach effect chain to item |

Description values may contain primitives, other descriptions, and homogeneous typed lists. They do
not expose canonical field names, arbitrary string keys, backend flags, or generic parameter maps.
Fluent methods return immutable replacement values. Graph operations retain explicit ownership,
stable keys, source provenance, effect, stage, and operand-axis contracts.

Generic plugin effects are separate from the core algebra. A plugin descriptor must pin its schema,
implementation identity, typed parameter schema, determinism class, supported backends, and digest.
The former `effect_type + map<string, value>` model has been removed from both executable Core and
the closed canonical schema. The typed plugin contract proves this boundary with
`plugin_reference_monochrome_v1()` as static topology
and the typed `video_plugin_scalar_effect(...)` constructor. Canonical IR carries a closed effect variant
and its content-addressed descriptor digest;
IR validation and FFmpeg 8 preflight require the exact descriptor schema and registered adapter.

The normative v6 Surface names and signatures are fixed by the
[Executable Standard Library v6](../language-design/executable-stdlib-v6/README.md) contract.

## Migration Rule

Migration was vertical and family-sized:

1. Add closed Domain types and operations to the unreleased v7 registry.
2. Add Surface resolution, Core verification, bounded runtime evaluation, and rollback tests.
3. Lower the frozen graph directly to exact canonical IR.
4. Prove canonical planner and backend behavior with typed fixtures and observable tests.
5. Add source-index/source-edit coverage and a Chinese executable example with observable evidence.
6. Remove the superseded parser, text expansion, and lowering route.

No entity may mix frontend ownership. Generated source text, Surface AST interpretation,
canonical JSON property projection, and reparsing are forbidden migration shortcuts.

## Temporal Programs

Canonical schema v10 owns backend-neutral, closed temporal DAGs. A program declares typed inputs,
topologically ordered typed nodes, one result, a content digest, and typed provenance. Bindings select
an owner, clock, declared typed parameters, and expected result type. Media properties store
binding IDs, never expression strings.

Analyzer results enter executable source through typed Build-stage `input analysis` declarations and
become concrete before Temporal residualization. Schema v10 deliberately has no time-varying analysis
input: such a contract would also need a pinned analyzer identity, sampling clock, interpolation,
content transport, and random-access rules. Unknown `analysis` input variants and `analyses` binding
fields are rejected instead of publishing a backend-incomplete digest placeholder.

Core v10 evaluates concrete pure operations and residualizes pure operations that depend on Temporal
inputs. `GraphEmit` and `LocalMutation` never residualize. Temporal values may flow only to explicitly
approved leaf operands; entity existence, kind, identity, owner, order, collection shape, resource
selection, and control flow that changes topology remain no later than Build.

Required clocks are sequence time, clip time, source time, frame, and normalized progress. Required node
families are typed constants and inputs, unary/binary arithmetic, comparison, select, curve sample,
vector/color construction and projection, and the closed pure builtin set. Programs share the graph
transaction budget and are discarded atomically on any failure.

Reusable component animation uses `animate property on target(owner, selectors...) { body }` as a
Build-stage GraphEmit expression that returns the owner handle. The attachment stores no string path:
freeze resolves the typed Item or Apply handle to the same absolute sink catalog used by root temporal
declarations. Its body is a verified Pure closure with typed implicit clocks, and residualization reads
that verified Core directly rather than generating or reparsing Surface source.

The render plan carries only reachable programs and bindings. Backends compile the same validated DAG;
a backend-neutral reference evaluator defines random-access semantics and drives differential tests.

## Production Gate

All checked-in examples use executable `main(Context) -> Project`; frontend auto-classification and text
expansion are absent. Component behavior is expressed by modules, functions, nominal values, closures,
typed slots and ordinary graph construction, not by a parallel macro language.

Every mechanism catalog row must point to a Chinese example and typed semantic evidence. Heavy full
FFmpeg example rendering stays outside ordinary CI; focused pixel/audio/metadata E2E targets remain
available and CI runs parsing, execution, canonical, plan, bundle, fixture, and evidence contracts.

Cache and artifact identities cover exact source graph bytes, declared inputs, Core version, domain and
temporal opset identities, canonical schema, planner, backend compiler, and runtime fingerprints.

All new language code must keep line and function coverage above 95%, all controlled files remain
strictly below 200 lines, and no gate may weaken canonical validation, source transaction atomicity,
centered true-overlap transitions, deterministic budgets, or source-of-truth editing.
