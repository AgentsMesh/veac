# Executable Standard Library v6

Status: Implemented normative v6 Surface contract.

## Purpose

This directory fixes the complete executable video-domain API before numeric operation IDs are
allocated. It is the machine-facing contract for Surface resolution, typed HIR, Core verification,
runtime graph construction, source indexing, and canonical lowering. Canonical JSON is an output of
this API, never an authoring model.

The API is a closed algebra. Every choice has a named descriptor type and constructor. Descriptors
are immutable and opaque. Aggregate methods attach a whole semantic value to its owner; there are no
field setters, object literals, generic parameter maps, backend option bags, or string discriminators.

## Signature Notation

```text
name(arg: Type, ...) -> Result
Receiver.method(arg: Type, ...) -> Result
```

All arguments are positional and evaluated left to right. A callable name has exactly one signature:
v6 has no overloading by arity, operand type, or return context. A method receiver is part of its
identity. `list<T>` is the homogeneous immutable Core list; every other capitalized name is a closed
DomainType in this contract.

Unless a section says `GraphEmit`, a listed constructor is `Pure` and creates an immutable descriptor.
Container constructors and owner methods are `GraphEmit`, execute only at Build stage, and return a
new handle in the same transaction. No graph handle is serializable or valid outside that transaction.
Every owned attachment boundary has singular and ordered-list methods; list attachment is atomic and
does not weaken static topology, single ownership, stale-handle, stage, or transaction rules.

Each callable receives one numeric v6 domain operation before implementation. The generated language
spec pins name, signature, receiver, result, effect, stage, topology operands, leaf operands, and
registry digest. This document deliberately does not guess numeric IDs.

## Families

- [Shared values and animation](values-animation.md)
- [Project, timeline, and resources](settings-materials.md)
- [Sources, time mapping, and generators](sources-generators.md)
- [Visual composition and masks](visual-masks.md)
- [Color pipelines and effects](color-effects.md)
- [Text and captions](text.md)
- [Audio processing and routing](audio.md)
- [Relations and adjustment applies](relations-apply.md)
- [Multicam, templates, and annotations](multicam-template-annotation.md)
- [Video delivery](delivery-video.md)
- [Auxiliary and adaptive delivery](delivery-auxiliary.md)
- [ABI, ownership, and lowering invariants](invariants.md)

## Boundary

Surface names are standard-library symbols, not lexer keywords. User modules may wrap them in
functions, methods, nominal values, closures, and bounded collection algorithms. They may not replace
root standard symbols. Pure functions may calculate descriptor operands; only verified `GraphEmit`
operations create topology.

The v6 Surface covers the full canonical editing model, except host probe snapshots, project revision,
applied-operation history, arbitrary metadata, and backend-private flags. Those are host, transaction,
or planner facts rather than authored video semantics. Plugin effects use separately pinned, typed,
content-addressed descriptors and registered backend adapters; arbitrary plugin names, generic
parameter maps, and backend filter strings never enter executable Core.
