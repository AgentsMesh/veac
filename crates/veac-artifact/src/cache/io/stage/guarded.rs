use std::io::Write;

use super::*;

impl CacheStage {
    pub(in crate::cache::io) fn seal_while(
        mut self,
        expected: &ExpectedContent<'_>,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        if let Err(primary) = self.verify_unsealed(expected, &mut guard) {
            return Err(self.cleanup_after(primary));
        }
        checked(&mut guard)?;
        let marker = match BoundFile::create(self.directory.file(), state::SEALED.as_ref()) {
            Ok(marker) => marker,
            Err(primary) => return Err(self.cleanup_after(primary)),
        };
        self.sealed = true;
        self.finish_commit(marker, expected, &mut guard)
            .map_err(ArtifactError::committed)
    }

    fn finish_commit(
        &mut self,
        marker: BoundFile,
        expected: &ExpectedContent<'_>,
        guard: &mut impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        step(guard, || marker.sync())?;
        step(guard, || self.directory.sync())?;
        step(guard, || self.directory.verify())?;
        if self.directory.entries(5)? != state::committed_entries() {
            return corrupt("sealed cache directory changed during commit");
        }
        if marker.size()? != 0 {
            return corrupt("cache commit marker changed during commit");
        }
        marker.verify_while(self.directory.file(), &mut *guard)?;
        self.verify_content_and_files(expected, guard)?;
        step(guard, || self.directory.sync_parent())
    }

    fn verify_unsealed(
        &mut self,
        expected: &ExpectedContent<'_>,
        guard: &mut impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        step(guard, || {
            self.descriptor.file_mut().flush().map_err(Into::into)
        })?;
        step(guard, || {
            self.payload.file_mut().flush().map_err(Into::into)
        })?;
        step(guard, || self.record.file_mut().flush().map_err(Into::into))?;
        step(guard, || self.descriptor.sync())?;
        step(guard, || self.payload.sync())?;
        step(guard, || self.record.sync())?;
        step(guard, || self.directory.sync())?;
        step(guard, || self.directory.verify())?;
        step(guard, || self.require_data_entries())?;
        self.verify_content_and_files(expected, guard)
    }

    fn verify_content_and_files(
        &mut self,
        expected: &ExpectedContent<'_>,
        guard: &mut impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        self.descriptor
            .verify_while(self.directory.file(), &mut *guard)?;
        self.payload
            .verify_while(self.directory.file(), &mut *guard)?;
        self.record
            .verify_while(self.directory.file(), &mut *guard)?;
        let descriptor = self.descriptor.read_bounded_while(
            MAX_ARTIFACT_METADATA_BYTES,
            "artifact descriptor exceeds the metadata limit",
            &mut *guard,
        )?;
        let record = self.record.read_bounded_while(
            MAX_ARTIFACT_METADATA_BYTES,
            "artifact record exceeds the metadata limit",
            &mut *guard,
        )?;
        if descriptor != expected.descriptor || record != expected.record {
            return corrupt("cache transaction metadata changed before commit");
        }
        let (digest, size) = self
            .payload
            .fingerprint_while(MAX_ARTIFACT_PAYLOAD_BYTES, &mut *guard)?;
        if &digest != expected.payload_digest || size != expected.payload_size {
            return corrupt("cache transaction payload changed before commit");
        }
        step(guard, || self.directory.verify())?;
        self.descriptor
            .verify_while(self.directory.file(), &mut *guard)?;
        self.payload
            .verify_while(self.directory.file(), &mut *guard)?;
        self.record.verify_while(self.directory.file(), guard)
    }
}

fn step(
    guard: &mut impl FnMut() -> bool,
    operation: impl FnOnce() -> ArtifactResult<()>,
) -> ArtifactResult<()> {
    checked(guard)?;
    operation()?;
    checked(guard)
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}
