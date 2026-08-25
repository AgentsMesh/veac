use std::sync::Arc;

use super::lifecycle::QueryEpoch;
use super::CompilerDatabase;
use crate::program::expression::{
    lower_function_batch, prepare_function_batch, CompiledFunctionBatch, ExpressionContext,
    ExpressionError, FunctionDefinition, FunctionQueryKey, TypedFunctionBatch,
};

impl CompilerDatabase {
    pub(crate) fn compile_functions(
        &self,
        context: &ExpressionContext,
        definitions: &[FunctionDefinition],
        retained_limit: usize,
    ) -> Result<ExpressionContext, ExpressionError> {
        let epoch = self.query_epoch();
        let key = FunctionQueryKey::new(context, definitions);
        if !context.static_values().is_empty() {
            self.bypass_semantic_caches();
            let batch = prepare_function_batch(context, definitions)?;
            let lowered = lower_function_batch(context, &batch, retained_limit)?;
            return Ok(context.clone().with_functions(lowered.functions().clone()));
        }
        let mut hir = self.cached_hir(&epoch, &key);
        if let Some(batch) = self.cached_core(&epoch, &key) {
            if batch.retained_bytes() > retained_limit {
                if hir.is_none() {
                    hir = Some(Arc::new(prepare_function_batch(context, definitions)?));
                }
                lower_function_batch(context, hir.as_ref().unwrap(), retained_limit)?;
            }
            return Ok(context.clone().with_functions(batch.functions().clone()));
        }
        let hir = match hir {
            Some(batch) => batch,
            None => {
                let batch = Arc::new(prepare_function_batch(context, definitions)?);
                let bytes = batch.retained_bytes();
                self.cache_hir(&epoch, key.clone(), bytes, batch)
            }
        };
        let lowered = Arc::new(lower_function_batch(context, &hir, retained_limit)?);
        let bytes = Some(lowered.cache_retained_bytes());
        let lowered = self.cache_core(&epoch, key, bytes, lowered);
        Ok(context.clone().with_functions(lowered.functions().clone()))
    }

    pub(super) fn cached_hir(
        &self,
        epoch: &QueryEpoch,
        key: &FunctionQueryKey,
    ) -> Option<Arc<TypedFunctionBatch>> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        lifecycle
            .admits(epoch)
            .then(|| self.hir.lock().expect("HIR cache lock poisoned").get(key))?
    }

    pub(super) fn cached_core(
        &self,
        epoch: &QueryEpoch,
        key: &FunctionQueryKey,
    ) -> Option<Arc<CompiledFunctionBatch>> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        lifecycle
            .admits(epoch)
            .then(|| self.core.lock().expect("Core cache lock poisoned").get(key))?
    }

    pub(super) fn cache_hir(
        &self,
        epoch: &QueryEpoch,
        key: FunctionQueryKey,
        bytes: Option<usize>,
        value: Arc<TypedFunctionBatch>,
    ) -> Arc<TypedFunctionBatch> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        let bytes = lifecycle.admits(epoch).then_some(bytes).flatten();
        self.hir
            .lock()
            .expect("HIR cache lock poisoned")
            .insert(key, bytes, value)
    }

    pub(super) fn cache_core(
        &self,
        epoch: &QueryEpoch,
        key: FunctionQueryKey,
        bytes: Option<usize>,
        value: Arc<CompiledFunctionBatch>,
    ) -> Arc<CompiledFunctionBatch> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        let bytes = lifecycle.admits(epoch).then_some(bytes).flatten();
        self.core
            .lock()
            .expect("Core cache lock poisoned")
            .insert(key, bytes, value)
    }

    fn bypass_semantic_caches(&self) {
        let _lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        self.hir.lock().expect("HIR cache lock poisoned").bypass();
        self.core.lock().expect("Core cache lock poisoned").bypass();
    }
}
