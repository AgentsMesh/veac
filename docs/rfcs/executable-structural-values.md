# RFC: Executable Structural Values And Bounded Iteration

Status: Implemented

## Scope

This document fixes the canonical surface and execution semantics for the G1
structural-value slice. Historical expression syntax is not a compatibility
constraint. The general Core, determinism, stage, effect, topology, provenance,
and atomic-ledger rules remain normative.

## Numeric Literals

An unsuffixed whole-number token is `int`; an unsuffixed token containing a
decimal point is exact `scalar`. A token with a unit suffix is the corresponding
exact dimension value regardless of whether it contains a decimal point.

```veac
1       // int
1.0     // scalar
1s      // time
1.5s    // time
```

`int` is signed 64-bit. Its unary negation, addition, subtraction, and
multiplication are checked. `int / int` produces an exact `scalar`, including for
non-integral results. Division by zero is an error. Other mixed numeric arithmetic
is rejected unless an existing unit rule explicitly defines it; authors use an
explicit conversion instead of implicit widening. Equality and ordering never
coerce `int` to `scalar`.

## Type Grammar

The canonical type forms are:

```text
primitive        int | scalar | time | length | percent | angle
                 | text | color | bool | identifier
list             list<T>
map              map<K, V> where K is text or identifier
tuple            (T, U, ...)
function         fn(T, U, ...) -> R effect pure|local|emit|any
nominal          Name or module.Name
```

Tuple types have at least two elements. Recursive structural types and recursive
nominal layouts are rejected. Types are fully resolved before Core lowering; Core
instructions refer to verified `TypeId`s rather than reparsing type text.

Each Core program owns a dense, duplicate-free type table. Public value types and
Core-internal linear token types share the table but are disjoint: builder tokens
cannot be returned, passed, captured, projected, or stored in a public value. The
verifier rejects malformed structural types before evaluation.

An immutable binding may state an expected type:

```veac
let selected: list<int> = [];
```

The annotation is checked, not cast. Expected types flow into empty literals and
closure checking. A non-empty literal is inferred locally and must still match any
expected type exactly.

## Structural Literals

Lists, maps, and tuples use one canonical form:

```veac
[1, 2, 3]
#{ "intro": 0s, "outro": 12s }
(1920, 1080)
```

A list is homogeneous and preserves source order. An empty list requires an
expected `list<T>` type. A tuple has fixed arity and heterogeneous element types;
parentheses without a comma remain grouping syntax.

A map key is `text` or `identifier`. Entries evaluate in source order, key before
value. A duplicate key fails immediately after its key is evaluated and its value
is not evaluated. Canonical storage and iteration sort keys by original UTF-8
bytes. An empty map requires an expected `map<K, V>` type.

Detached empty values retain their resolved type. Canonical value rendering emits
a typed pure block such as `{ let value: list<int> = []; value }`, so an empty
value and any structure containing it can be parsed again without ambient type
context.

Structural equality is recursive, deterministic, and type exact. List and tuple
equality follows element order. Map equality follows canonical key order. Values
are immutable and collection APIs never expose storage order other than the
specified iteration order.

## Ranges

Finite lazy ranges use these forms:

```veac
0 .. 10
10 .. 0 by -2
```

Range bounds and step are `int`; the default step is `1`. Ranges are half-open.
Zero step is an error. A step pointing away from the bound yields an empty range.
Count and arithmetic overflow are checked before iteration. Range construction is
non-associative and binds below additive arithmetic but above ordering.

## Typed Closures

Closures use function syntax in expression position:

```veac
fn(value: int, offset: int) -> int effect pure { value + offset }
```

Every parameter and return type is explicit. The body is a canonical block. A
closure captures only referenced immutable serializable locals. Module-static
constants are resolved before final closure compilation and embedded as typed
literals, so they do not become activation captures. Capture order for lexical locals
is first resolved occurrence. Typed HIR records capture IDs and Core closure
construction stores values in that order.

Closures are immutable values with structural function types. Equality, ordering,
serialization as a project value, dynamic dispatch, reflection, and recursive
self-capture are rejected. A closure may be called only where its exact function
type is known.

## Collection Operations

The initial closed operations are:

```veac
map(values, fn(value: T) -> U effect pure { ... })          // list<U>
filter(values, fn(value: T) -> bool effect pure { ... })    // list<T>
fold(values, initial, fn(acc: U, value: T) -> U effect pure { ... })
for value in values { ... }                     // list of body results
```

`map`, `filter`, and `fold` evaluate input, then closure arguments, exactly once
from left to right. `fold` is left-associative. `for` introduces one immutable
lexical local and preserves input order. The iterable is a list, a canonical map
entry sequence, or a finite range. There is no `while`, implicit parallelism,
lazy user generator, early control transfer, or user-defined iterator protocol.

Collection shape, range bounds, filter predicates, and iteration count must be at
most Build stage. A Temporal leaf may occur in a retained element only when it
does not affect shape, key, order, identity, resource selection, or topology.

## Budgets And Core

Before collection allocation or iteration, execution computes the complete finite
count and atomically reserves aggregate iterations, collection elements, and
logical bytes. A failed reservation allocates nothing and commits no ledger
dimension. Element evaluation still charges fuel and payload as it executes.

Typed HIR keeps literal order, lexical IDs, inferred structural types, spans,
captures, effects, stages, and dependency masks. Verified Core adds explicit
instructions for collection construction, closure construction/call, range count,
and bounded iteration. Loop headers use block parameters; no Surface node or
runtime string lookup participates in execution.

Map literals lower to a linear `begin -> key -> value -> finish` protocol. `begin`
reserves the complete entry count and container bytes before allocating private
builder storage. `key` consumes the previous builder token, checks all prior keys,
and produces one pending token. A duplicate fails there, before its value executes.
`value` consumes that pending token and `finish` consumes the final builder. Tokens
are single-use, ordered, and unobservable; private builder mutation remains `Pure`.

Collection bytes account for newly allocated container slots and handles. Payload
already owned by an element handle is not charged recursively a second time;
nested collection construction and original text payload allocation have their own
reservations. Count and slot multiplication are checked before allocation.

Pure iteration returns a value. Graph-emitting iteration is a distinct instruction
and requires a stable logical key for each entity before emission. Duplicate keys
fail atomically for that emission and numeric loop indices remain provenance only.

## Conformance Gate

This slice is complete only when tests cover literal/type inference, empty-value
context, exact and overflow arithmetic, duplicate-map evaluation order, canonical
map ordering, range direction/count/overflow, closure capture order, nested lexical
shadowing, left-to-right higher-order execution, all ledger dimensions, stage
sinks, Core verification, serialization determinism, CLI diagnostics, and
source-edited function bodies. New production code must exceed 95% line coverage
and every controlled source file remains below 200 lines.
