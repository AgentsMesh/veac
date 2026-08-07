# RFC: Executable Core Semantics

Status: Implemented

## Scope

This document fixes the G1 language semantics behind the executable-language
architecture. Surface grammar may gain additive domain forms, but implementations
must preserve these rules. Historical `.veac` syntax is not a compatibility
constraint.

## Functions And Blocks

Functions use one canonical block form:

```veac
fn choose(value: time, enabled: bool) -> time {
    let padded = value + 250ms;
    if enabled && padded >= 2s { padded } else { 2s }
}
```

A block contains zero or more ordered `let`, `var`, or `set` statements followed by
one tail expression. `let` is immutable; `var` creates an activation-local typed slot;
`set` updates only an already declared `var` of the exact same type. A block has the
type and value of its tail. There is no implicit unit value and no trailing semicolon
after the tail. Initializers execute in source order and a name enters scope only
after its initializer succeeds.

A nested block may shadow an outer local. Local lookup precedes parameters and
declared externals. Each function or closure activation owns a fresh slot table that
is discarded on return or failure. Mutable locals cannot contain function values,
cross an activation boundary, or be captured by a closure. A function, method, entry,
or Temporal body may reference a module-static constant. Resolution closes direct and
transitive constant dependencies before final Core compilation, rejects cycles, and
embeds the typed value as a literal. This is not a runtime ambient capture. Closures
may additionally capture immutable local values explicitly in their typed environment.

Function arguments execute exactly once, from left to right. Recursion is rejected.
Unused declarations are still parsed, resolved, typed, effect-checked, and verified.
Every closure and function value declares `effect pure|local|emit|any`; verified Core
recomputes actual evidence and rejects an understated contract. Local mutation never
residualizes into a Temporal program.

## Operators And Branches

Precedence, highest to lowest, is unary, multiplicative, additive, ordering,
equality, logical AND, and logical OR. Ordering and equality chains are rejected;
authors must combine comparisons with a logical operator.

- `!` accepts only `bool`; unary `+` and `-` accept numeric values.
- `<`, `<=`, `>`, and `>=` require the same numeric type.
- `==` and `!=` require the same type and use deterministic structural equality.
- `&&` and `||` accept `bool` and short-circuit from left to right.
- Arithmetic retains exact unit-aware behavior and checked overflow.

`if` always has an `else`. Its condition is exactly `bool`, without truthiness.
Both branches are resolved and typed, but only the selected branch executes. The
branch result types must be identical; there is no implicit numeric coercion,
union, or nullable result.

## Types And Values

Primitive types are `int`, `scalar`, `time`, `length`, `percent`, `angle`, `text`,
`color`, `bool`, and `identifier`. `int` is signed 64-bit checked arithmetic and is
the only range/index type. Existing scalar and unit values retain exact rational
semantics.

Structural types are `list<T>`, `map<K, V>`, tuples, and
`fn(T...) -> R effect pure|local|emit|any`. The effect contract is part of function
type identity and is explicit at every nesting level. Lists are
ordered and homogeneous. Tuples have fixed heterogeneous arity. Map keys are
restricted initially to `text` and `identifier`; canonical iteration uses original
UTF-8 bytes in ascending order and duplicate keys are errors.

Nominal `struct` and closed `enum` identities include canonical source ID and
declared name. Struct field order is declaration order. Values are immutable after
construction. Recursive nominal value layouts are rejected until an explicit
bounded reference type exists.

Enum matching is exhaustive unless a final `_` arm is present. A pattern binds
only immutable values from its selected variant. Non-selected arms never execute.

## Collections And Iteration

Ranges are finite, lazy, half-open `int` ranges. A zero step is invalid. A step with
the wrong direction produces an empty range. Count and overflow are checked before
allocation or iteration.

`map`, `filter`, and `fold` visit elements exactly once from left to right. `fold`
is left-associative. A `for` expression preserves input order and returns a list;
it may carry the verified graph-emission effect described below. Hash order,
implicit parallelism, `while`, unbounded generators, and user defined ordering are
not language semantics.

Iteration count is reserved atomically before collection allocation. Nested loops
share one source-graph ledger. Effectful graph iteration additionally requires a
stable logical key for every emitted entity; duplicate keys are errors. Numeric
indices are provenance metadata, not entity identity.

## Objects, Methods, And Closures

`impl Type` declares statically resolved methods. Method dispatch is nominal and
closed; there is no inheritance, prototype lookup, reflection, or runtime method
table mutation. `self` is an immutable receiver and cannot be assigned through
method syntax.

Closure parameters, return types, and effect contracts are explicit. Core verification
derives effect evidence from the closure body and rejects contracts that understate it.
A closure captures only immutable
serializable locals that its body references. Capture order follows first resolved
symbol occurrence and is recorded in Typed HIR. A closure cannot capture ambient
I/O, mutable graph builders, module loaders, or temporal topology controls.

## Effects And Stages

Effect and stage are orthogonal. The initial effect lattice contains `Pure`,
`LocalMutation`, and `GraphEmit`; effects retain both their maximum summary and
independent mutation evidence. Local mutation cannot leak shared mutable identity.

The stage lattice is `Const < Build < Temporal`. Stages are inferred per expression
from dependencies, not declared on functions. A pure function may constant-fold,
execute during build, or residualize when called with temporal inputs.

Aggregate shape and leaf stage are tracked separately. List size, map keys, range
bounds, filter predicates, match variants controlling shape, entity kind, identity,
ordering, resource selection, and module selection must be no later than `Build`.
Allowed media-property leaves may be `Temporal` and residualize.

The invariant is `static topology, dynamic leaf values`. A temporal dependency that
can create, remove, reorder, rename, or retarget a project entity is a type/stage
error before build execution.

## Determinism And Failure

All evaluation order in this document is observable through errors and budgets.
Equivalent source graphs and declared inputs produce byte-identical envelopes.
There is no ambient filesystem, network, process, environment, clock, thread,
reflection, dynamic import, or implicit randomness capability.

Every resource reservation is checked and atomic. A failed expression, function,
loop, graph build, residualization, or edit publishes no partial scope, project,
canonical envelope, cache entry, or source rewrite.

Diagnostics retain definition span, call-site span, call frames, lexical binding
span, loop logical key, and generated entity provenance. Source edits always target
typed `.veac` fragments and rebuild canonical IR; generated IR is never decompiled.
