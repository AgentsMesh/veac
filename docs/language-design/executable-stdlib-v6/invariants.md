# ABI, Ownership, And Lowering Invariants

## Closed Surface

The files in this directory are one contract. A v6 implementation is conforming only when every
listed DomainType and callable is present and no production callable bypasses its contracts.

- Each constructor name has exactly one positional signature and one result type.
- Each method name has exactly one signature for its receiver type.
- Variant selection is a typed constructor, never text, integer tags, map keys, or return context.
- Opaque graph containers and descriptors have no field projection or structural construction.
- No callable accepts JSON, `map<text, value>`, generic effect parameters, metadata, or backend flags.
- Closed absence types are semantic variants; the Surface does not expose nullable canonical fields.
- Defaults are ordinary versioned library functions assembled from the same listed operations.

The generated `language-spec` is the machine ABI. For every callable it records the stable Surface
name, receiver, ordered operands, result, numeric domain operation ID, effect, latest stage, topology
operand indexes, leaf operand indexes, bounds, and canonical lowering rule. Core stores only numeric
operation IDs plus typed operands; runtime dispatch never uses Surface names.

## Effects

All descriptor constructors are `Pure`. The following operation classes are `GraphEmit`:

```text
project, sequence, four layer constructors, item
multicam_group, annotation, delivery, apply
Project.with_resource, Project.with_sequence, Project.entry
Project.with_multicam_group, Project.with_annotation, Project.with_delivery
Sequence.with_layer, Sequence.with_relation, Sequence.with_apply
Layer.with_item
Project.with_resources, Project.with_sequences, Project.with_multicam_groups
Project.with_annotations, Project.with_deliveries
Sequence.with_layers, Sequence.with_relations, Sequence.with_applies, Layer.with_items
Item.with_visual, Item.with_audio, Item.with_effect, Item.with_template
```

If implementation splits a class into multiple numeric operations, every split keeps the documented
Surface signature and effect. No listed operation is `LocalMutation`. Immutable handles do not turn a
graph operation into Pure work.

## Ownership

```text
Project owns Resource, Sequence, MulticamGroup, Annotation, Delivery.
Sequence owns Layer, Relation, Apply.
Layer owns Item.
Item owns attached VisualStyle, AudioStyle, Effect chain, and TemplateContract.
```

Descriptors may retain typed non-owning references only within the same graph transaction. A handle has
one owner, cannot be attached twice, cannot form a cycle, and cannot cross a module execution, declared
input, cache boundary, map key, or published value. Keys are explicit topology operands and unique in
their documented owner scope. Ordinal is provenance and order, never identity.

Every owned boundary exposes singular and plural attachment. A plural call preserves list order and
validates the complete list before assigning any owner or extending the parent. Empty lists are valid
identity updates; any invalid child rolls back the whole graph transaction.

`Project.entry` references exactly one already owned Sequence. Relation endpoints, Apply targets,
annotation targets, source references, font/LUT references, delivery mix sources, and template targets
must be reachable from the same closed Project.

## Stage And Operand Axes

Topology operands must be no later than Build stage:

```text
entity kind, key, owner, target, reference, order, list shape, enabled entity existence
resource/source selection, stream selection, relation endpoint, Apply target, delivery artifact kind
multicam angle/switch membership, template slot kind, annotation target/payload variant
```

Approved Temporal leaves are typed animation samples and numeric/color/style parameters that do not
change graph reachability or collection shape. They include transform, crop, opacity, mask motion/edge,
effect curves, text reveal/transform/opacity, audio gain/pan, source-time samples, and Apply mix opacity.
Static codec, routing, identity, stream, effect-kind, LUT-resource, and delivery choices cannot be
Temporal even if their canonical representation is a scalar or string.

G2 accepts constant/keyframe animation descriptors. G3 may residualize approved leaf inputs to the
closed temporal DAG described by the completion RFC. It must retain the same DomainType at the sink;
expression strings, Surface AST callbacks, and dynamic operation lookup remain forbidden.

## Transaction And Budgets

One `main(Context) -> Project` execution owns one arena. Receiver and operands evaluate left to right.
Before mutation, every GraphEmit reserves its entity and emitted-byte delta. Bounded GraphEmit `map`
reserves collection fan-out before its first callback. A type, validation, ownership, budget, or later
execution failure discards the complete arena and publishes no project, source index, provenance, or
cache artifact.

Core verification checks operation identity, operand/result types, effect, stage, declared bounds,
handle affinity, and topology/leaf flow before evaluation. Runtime repeats dynamic ownership and exact
budget checks; canonical validation is still mandatory after freeze.

## Canonical Lowering

Freeze walks the connected graph in stable owner/key order, materializes documented neutral choices,
resolves handles to canonical IDs, and directly constructs the current canonical model. It never:

- formats `.veac` and reparses it;
- interprets Surface AST or invokes an authoring frontend;
- projects descriptors through JSON field names;
- infers a variant from a filename, arbitrary string, or backend;
- silently drops an unsupported descriptor or incompatible combination.

Probe snapshots, revision, applied operations, schema envelope fields, and planner/backend facts are added
by their owning boundary after authored graph freeze. Closed typed `authorship` records are lowered directly
from verified graph provenance; they are not hidden constructor operands or a metadata property bag.

## Conformance Evidence

Each operation requires contract tests for valid construction, wrong operand/result corruption, effect
and stage corruption, one-short/exact budgets, duplicate ownership/key, stale/cross-graph handle, and
transaction rollback. Each closed variant requires canonical lowering and source-index/source-edit
evidence. Family integrations must pass executable source through Core verification, evaluation,
canonical validation, plan/bundle preservation, and focused backend evidence.

New language code keeps line and function coverage above 95 percent. Controlled files remain strictly
below 200 lines. Heavy all-example FFmpeg rendering stays outside ordinary CI; focused visual, audio,
caption, metadata, transition-overlap, and delivery E2E tests remain required gates.
