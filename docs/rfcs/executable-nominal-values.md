# RFC: Executable Nominal Values

Status: Implemented

## Scope

This document fixes the first executable semantics for nominal `struct`, closed
`enum`, exhaustive `match`, and statically resolved `impl` methods. It refines the
nominal-value rules in `executable-core-semantics.md`. Historical source syntax is
not a compatibility constraint.

Nominal values are immutable data. They are not property bags, component schemas,
or a second graph authoring language. Graph construction is the separate domain-value
and `GraphEmit` concern defined by the current executable registry.

## Identity And Registry

A nominal `TypeId` is a 32-byte, domain-separated SHA-256 digest of the canonical
source ID and declared type name. Field, variant, method, import alias, and source
content changes do not change that identity. A separate definition digest covers
the complete resolved layout and detects stale or corrupted contracts.

Each source graph owns one immutable type registry. Imports add visible aliases but
retain the defining `TypeId`. A private type is visible inside its defining module
and is never copied into an importing scope. Duplicate visible names, duplicate
identities with different definitions, and unknown type references are errors.

Declarations are collected before layouts are resolved, so forward references are
valid. A cycle through any structural nesting is rejected. This includes cycles
through list, map, tuple, range, function parameter, and function result types.

## Declarations

The canonical forms are:

```veac
export struct Brand {
    color: color,
    title_size: length,
}

export enum Placement {
    Center,
    Corner { x: length, y: length },
}
```

Struct fields and enum variants are ordered by declaration. Variant payload fields
are named and ordered. A trailing comma is accepted. Duplicate fields or variants,
empty names, excessive arity, and unresolved field types are rejected at their
authored spans.

Struct and payload construction uses named fields:

```veac
Brand { title_size: 64px, color: #ffffffff }
Placement.Center
Placement.Corner { x: 24px, y: 32px }
```

Field initializers execute exactly once in authored order. The resulting immutable
storage is reordered to declaration order. Missing, duplicate, unknown, or
wrongly-typed fields are compile errors. A unit variant cannot receive a payload,
and a payload variant cannot omit its fields.

## Members And Methods

`.` is postfix member syntax. A lexical binding in the first segment shadows a
same-named namespace. Otherwise resolution chooses the longest exact visible
module-qualified value, function, or type prefix, then resolves remaining segments
from left to right as projections or methods. This preserves imported names while
also supporting projections from arbitrary expressions.

```veac
brand.color
make_brand().title_size
timing.default_duration
```

Methods use one closed nominal receiver:

```veac
impl Brand @presentation {
    export fn title(self, value: text) -> text { value }
}

brand.title("Opening")
```

`self` is the first immutable parameter and has the enclosing nominal type. Method
calls evaluate the receiver first, then explicit arguments once from left to right.
They lower to a statically selected `FunctionId` with the receiver as argument zero.
Method visibility is explicit: `export fn` enters importing scopes and plain `fn`
remains private. There is no inheritance, extension of foreign types, runtime
dispatch, reflection, method-table mutation, or implicit receiver conversion. A
method body may call private functions and methods from its defining module through
the normal captured registry contract.

## Matching

The canonical match form is:

```veac
match placement {
    Placement.Center => 0px,
    Placement.Corner { x, y: vertical } => x + vertical,
}
```

A pattern names a variant of the scrutinee type. Payload shorthand binds a field to
the same lexical name; `field: binding` renames it. Bindings are immutable and are
visible only in that arm. Duplicate bindings, missing or unknown payload fields,
and variants from another enum are errors.

Every variant must appear exactly once unless a final `_` arm covers all remaining
variants. A wildcard before another arm is invalid. All arm result types are
identical. Every arm is parsed, resolved, typed, effect-checked, and verified, but
only the selected arm executes. Surface wildcard arms lower to explicit remaining
Core arms, so verified Core is always exhaustive.

## Core Contract

Core carries a dense, immutable nominal definition table. Value types refer to a
known `TypeId`; constructors and projections use dense field and variant indices.
Core provides struct construction, struct projection, enum construction, and an
exhaustive enum-match terminator. Match targets receive variant payload values as
typed block parameters.

The verifier independently rejects unknown identities, definition-digest mismatch,
non-dense indices, duplicate or missing match variants, invalid target parameter
types, wrong constructor operands, wrong projection results, malformed control
flow, and non-dominating values. Runtime executes only verified Core and never
looks up a field, variant, type, or method by source string.

## Stages, Effects, And Budgets

Nominal construction and projection are `Pure`. Their fixed layout is Const shape;
field payloads contribute leaf stage and dependencies. An enum discriminant is
shape. A Temporal discriminant cannot select a branch that changes collection
shape, graph topology, identity, ordering, resource selection, or another Build
sink. Allowed Temporal payload leaves residualize through authored `animate` declarations.

Before allocating nominal storage, execution atomically reserves the container,
field handles, and any new payload owned by the expression. Existing immutable
payload handles are not charged recursively a second time. Failed construction or
matching publishes no partial value, ledger reservation, graph, cache entry, or
source edit.

## Source And Canonical Boundaries

Source indexes retain declaration, field, variant, method, call-site, match-arm,
and pattern-binding spans. Source edits modify typed `.veac` fragments and rebuild
the source graph. Canonical IR stores stable identities, resolved layouts, values,
and provenance; it never stores unchecked Surface AST or decompiles nominal values
back into source.

## Conformance Gate

Completion requires unit, integration, module, CLI, source-edit, verifier-corruption,
budget-boundary, deterministic-ID, and canonical round-trip tests. Tests must cover
forward and imported references, private visibility, field evaluation order,
arbitrary postfix projection, exhaustive and wildcard match, non-selected-arm
failure isolation, static method selection, nested nominal values, recursive-layout
rejection, stage sinks, and malformed raw Core. New language code must exceed 95%
line coverage and every controlled source file must remain below 200 lines.
