use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use veac_artifact::ContentDigest;
use veac_ir::ImageSequencePattern;

use crate::executor::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

#[derive(Default)]
pub(in crate::executor) struct FakeFfmpeg {
    pub calls: RefCell<Vec<Vec<String>>>,
    pub deadlines: RefCell<Vec<Instant>>,
    pub fingerprint_calls: Cell<usize>,
    pub encoder_calls: Cell<usize>,
    pub muxer_calls: Cell<usize>,
    pub decoder_calls: Cell<usize>,
    pub demuxer_calls: Cell<usize>,
    pub filter_calls: Cell<usize>,
    pub hardware_backend_calls: Cell<usize>,
    pub hardware_device_calls: Cell<usize>,
    pub encoders: BTreeSet<String>,
    pub muxers: BTreeSet<String>,
    pub decoders: BTreeSet<String>,
    pub demuxers: BTreeSet<String>,
    pub filters: BTreeSet<String>,
    pub hardware_backends: BTreeSet<String>,
    pub hardware_devices: BTreeSet<String>,
    pub fail_on_call: Option<usize>,
    pub mutate_on_call: Option<(usize, PathBuf, Vec<u8>)>,
}

impl FfmpegEnvironment for FakeFfmpeg {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        self.fingerprint_calls.set(self.fingerprint_calls.get() + 1);
        Ok(FfmpegFingerprint {
            version: "ffmpeg version fake-1".to_owned(),
            configuration: ContentDigest::sha256(b"fake-configuration"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.encoder_calls.set(self.encoder_calls.get() + 1);
        Ok(self.encoders.clone())
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.muxer_calls.set(self.muxer_calls.get() + 1);
        Ok(self.muxers.clone())
    }

    fn decoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.decoder_calls.set(self.decoder_calls.get() + 1);
        Ok(self.decoders.clone())
    }

    fn demuxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.demuxer_calls.set(self.demuxer_calls.get() + 1);
        Ok(self.demuxers.clone())
    }

    fn filters(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.filter_calls.set(self.filter_calls.get() + 1);
        Ok(self.filters.clone())
    }

    fn hardware_backends(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.hardware_backend_calls
            .set(self.hardware_backend_calls.get() + 1);
        Ok(self.hardware_backends.clone())
    }

    fn hardware_devices(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.hardware_device_calls
            .set(self.hardware_device_calls.get() + 1);
        Ok(self.hardware_devices.clone())
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        self.deadlines.borrow_mut().push(invocation.deadline());
        let arguments = invocation.arguments();
        self.calls.borrow_mut().push(arguments.to_vec());
        let call = self.calls.borrow().len();
        if self.fail_on_call == Some(call) {
            return Err(RuntimeError::new("fake FFmpeg failure"));
        }
        if let Some(prefix) =
            option(arguments, "-passlogfile").filter(|_| option(arguments, "-pass") == Some("1"))
        {
            fs::write(format!("{prefix}-0.log"), b"passlog").unwrap();
            fs::write(format!("{prefix}-0.log.mbtree"), b"tree").unwrap();
            self.mutate(call);
            return Ok(());
        }
        let output = arguments.last().expect("output argument");
        let output_path = Path::new(output);
        let pattern = output_path
            .file_name()
            .and_then(|value| value.to_str())
            .and_then(ImageSequencePattern::parse);
        if let Some(pattern) = pattern {
            fs::write(
                output_path.with_file_name(pattern.format_index(1)),
                b"frame-one",
            )
            .unwrap();
            fs::write(
                output_path.with_file_name(pattern.format_index(2)),
                b"frame-two",
            )
            .unwrap();
        } else {
            fs::write(output, b"rendered-output").unwrap();
        }
        self.mutate(call);
        Ok(())
    }
}

impl FakeFfmpeg {
    fn mutate(&self, call: usize) {
        if let Some((expected, path, content)) = &self.mutate_on_call {
            if *expected == call {
                fs::write(path, content).unwrap();
            }
        }
    }
}

fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}
