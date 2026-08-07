# Executable Source To Canonical IR

VEAC source executes into canonical ProjectEnvelope schema version 9 with minimum reader 9. Mapping is a
typed compiler boundary, not a field-by-field textual projection.

```text
Surface declarations
  -> typed HIR -> verified Core v10
  -> bounded graph transaction -> frozen Domain graph
  + authored temporal residualization
  -> direct ProjectEnvelope
  -> canonical validation
```

## Identity And Ownership

Source `identifier` values and owner attachment calls determine logical paths. Lowering derives prefixed
canonical IDs using length-framed stable identities. Handles carry graph affinity and Domain type; an Item
cannot be attached to two Layers, a Resource cannot be confused with a Sequence, and no raw text reference
is resolved during backend planning.

Project owns resources, sequences, multicam groups, annotations and deliveries. Sequence owns layers,
relations and Apply pipelines. Layer owns Items. Delivery owns typed artifacts. Lowering rejects disconnected
or duplicate ownership before producing JSON.

## Value Mapping

- time/unit literals become exact rational or dimensioned canonical values;
- closed enum constructors become tagged canonical variants;
- ordered lists remain ordered where order is semantic;
- maps are canonicalized by original UTF-8 key bytes;
- optional state is represented by a typed choice constructor, never null-by-accident;
- `Animatable<T>` is constant, keyframes or a typed Temporal binding ID;
- local file locators remain source-root-relative and carry declared content identity.

There is no generic object-to-map conversion and no unknown-field preservation. Standard-library operation
IDs choose explicit lowering functions.

## Temporal Mapping

An authored `animate` declaration supplies one closed sink, stable logical target, verified Core expression
and typed clock identities. Residualization emits a canonical DAG, binding, provenance and deterministic
IDs. The sink stores only the binding ID. Backend code never sees Surface expression text.

## Canonical Edit Boundary

Canonical edits are explicit revisioned operations. They do not update `.veac`:

```json,canonical-edit-batch
{
  "operation_id": "op_disable_host",
  "base_revision": 0,
  "atomic": true,
  "preconditions": [],
  "operations": [
    {
      "type": "set_clip_enabled",
      "clip_id": "itm_host-shot",
      "enabled": false
    }
  ]
}
```

`SourceEditBatch` is the separate source-of-truth boundary. It targets module-qualified declarations and
body sites, rebuilds the program, and never reconstructs source from this JSON.

## Backend Boundary

`veac-ir` validates schema, ownership, references, temporal reachability, true-overlap transitions, delivery
contracts and budgets. `veac-plan` resolves media/probe facts. `veac-codegen` emits a typed BackendBundle.
`veac-runtime` executes only bundle tasks and verifies outputs. None of those layers define Surface language
semantics or accept an unvalidated property bag.
