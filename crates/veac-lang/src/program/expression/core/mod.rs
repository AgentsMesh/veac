mod compiled;
mod digest;
mod id;
mod metadata;
mod model;
mod registry;
mod retained;
mod type_table;
mod verify;

pub use compiled::{CompiledExpression, CompiledFunction};
pub(crate) use digest::{closure_digest, input_declarations_digest};
pub use id::{
    BlockId, ClosureDefinitionId, CoreDigest, CoreInputDeclarationDigest, CoreTypeId, FunctionId,
    InputId, LocalSlotId, ValueId,
};
pub(crate) use metadata::operand_metadata;
#[cfg(test)]
pub(crate) use metadata::EffectEvidence;
pub use metadata::{CoreValueMetadata, DependencyMask, Effect, FunctionSummary, Stage};
pub use model::{
    ArithmeticOperator, ComparisonOperator, CoreBlock, CoreBlockParameter, CoreBuildInputId,
    CoreCallTarget, CoreCallableInput, CoreClosureDefinition, CoreForEach, CoreForEachEffect,
    CoreForEachOrder, CoreForEachProvenance, CoreForEachSlot, CoreForEachSlotId, CoreInput,
    CoreInputIdentity, CoreInstruction, CoreInstructionKind, CoreLocalSlot, CoreMatchArm,
    CoreNominalDefinition, CoreProgram, CoreTemporalComposeOperation, CoreTemporalInputIdentity,
    CoreTemporalProjectOperation, CoreTerminator, CoreUnaryOperator, EqualityOperator,
    CORE_VERSION,
};
pub use type_table::{CoreType, CoreTypeEntry, CoreTypeTable};

pub(crate) use registry::FunctionRegistry;
pub(crate) use verify::{
    verify, verify_with_input_trust, VerifiedClosureDefinition, VerifiedCoreProgram,
};

#[cfg(test)]
pub(super) mod tests;
