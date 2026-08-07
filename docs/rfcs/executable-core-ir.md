# RFC: Executable Core IR

Status: Implemented

## Boundary

Executable Core IR is the only interpreter input. Surface tokens, parser nodes, and
Typed HIR are compiler data and never cross the runtime boundary. Core is versioned
independently from surface grammar and canonical project schema.

```text
surface AST -> resolved Typed HIR -> verified Core IR -> bounded evaluator
```

Typed HIR retains authored names, spans, nominal symbols, inferred types, effects,
stages, dependency masks, and topology constraints. Core replaces names and lexical
structure with stable IDs and explicit evaluation order.

## Identity And Storage

- `FunctionId` is derived from canonical source ID plus declaration identity.
- `BlockId` and `ValueId` are dense within one function/program definition.
- Nominal `TypeId` is derived from canonical source ID plus declared type name.
- Generated entities additionally carry stable authored or loop logical keys.

Visible module names map to IDs. A registry owns each function/type definition once;
qualified aliases add bindings and never copy code. Core call instructions contain
`FunctionId`, never source spelling or `Arc` call graphs.

Core v10 also embeds typed Build/Temporal input identities and the closed domain opset digest. Input identity
and declaration digest domains are version-separated as `veac.core-v10.*`; those values participate in closure
digests and must match across callers, callees, verifier, and runtime.

Variable-size payloads use immutable arena handles. Cloning a value copies a handle;
the execution ledger charges allocation once and evaluation work separately.

## Control-Flow Form

A Core function owns ordered blocks. Each block has typed parameters, ordered
instructions, and exactly one terminator. Values are immutable SSA-like IDs.

Initial instructions are:

- constants and declared-input loads;
- primitive unary, arithmetic, comparison, and equality operations;
- constructors and projections for tuple, list, map, struct, and enum values;
- direct function and method calls by `FunctionId`;
- closure construction and invocation with typed capture slots;
- bounded range and collection operations;
- domain-value construction and graph emission;
- temporal leaf construction for residualizable pure operations.

Initial terminators are `return`, `jump`, `branch`, `match`, and bounded `for-each`.
Jump arguments bind target block parameters. `branch` and `match` execute one target.
`for-each` declares maximum count, stable order, element/index slots, loop body, and
continuation; graph-emitting loops also carry the logical-key value.

Short-circuit operators lower to blocks and branches, never eager binary
instructions. Block locals disappear into `ValueId` references. Surface AST is not
retained to recover evaluation order.

## Types, Effects, And Dependencies

Every instruction result and block parameter has a closed Core type. Every
instruction records inferred effect, aggregate-shape stage, leaf stage, dependency
set, source provenance, and applicable topology sink policy.

Function summaries record parameter dependency masks, capture types, effects,
result shape/leaf stages, emitted graph kinds, and resource upper bounds. Calls
instantiate summaries with argument stages; functions are not manually labeled
`const`, `build`, or `temporal`.

## Verification

The verifier runs before evaluation and after canonical decode. It rejects:

- unknown or duplicate IDs, invalid block targets, and malformed terminators;
- use before definition, non-dominating values, and block argument mismatches;
- opcode/type mismatches, invalid calls/captures, and non-exhaustive enum matches;
- recursion and call-depth overflow;
- unbounded or overflowed iteration and allocation bounds;
- effect violations and temporal data reaching a topology sink;
- unsupported language/opset versions or backend-required operations;
- missing provenance, logical keys, declared inputs, or resource limits.

Verification is deterministic, bounded, and diagnostic-producing. It never executes
surface source or backend commands.

## Evaluation

The reference evaluator processes instructions and call arguments left to right.
It executes only the selected branch or match arm. All calls, blocks, loops, values,
emissions, and residual nodes debit one source-graph `ExecutionLedger`.

The ledger atomically reserves fuel, value arena bytes, collection elements/bytes,
iterations, emitted entities/bytes, residual nodes/bytes, diagnostics, and output.
Reservation precedes allocation. Failure aborts the graph-builder transaction and
cannot publish partial IR or cache state. Authored Temporal leaves receive one typed
view of that ledger; per-program builders own node IDs and canonical limits but cannot
own, clone, or reset source-graph counters.

Pure known instructions fold. Build instructions construct immutable domain values
and static graph topology. Pure temporal instructions produce canonical residual
nodes rather than evaluating a frame. Backends consume residual programs only after
canonical validation.

## Canonicalization

Runtime Core is not automatically the canonical interchange encoding. Canonical
project envelopes contain a frozen project graph, closed temporal programs, declared
input identities, language/opset versions, content digests, and provenance.

Any persisted Core encoding must use stable numeric opcodes, canonical field order,
closed tagged operands, explicit limits, and byte-for-byte round-trip tests. Rust
enum layout, pointer identity, hash iteration, and source AST serialization are not
part of the format.

## Migration Rules

Each staged compiler change must keep one path only:

1. Parse authored source into Surface AST.
2. Resolve and type/effect/stage-check into Typed HIR.
3. Lower all executable behavior into Core.
4. Verify Core.
5. Evaluate or residualize Core.
6. Validate the canonical envelope before backend lowering.

No production path may evaluate Surface AST, reparse generated source, bypass the
ledger, call a user function by string lookup, or let FFmpeg define language
semantics.
