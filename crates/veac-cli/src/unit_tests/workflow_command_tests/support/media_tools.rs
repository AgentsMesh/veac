use std::path::{Path, PathBuf};

#[cfg(unix)]
pub(in crate::unit_tests::workflow_command_tests) fn fake_ffmpeg(root: &Path) -> PathBuf {
    super::executable(
        root,
        "ffmpeg.sh",
        "#!/bin/sh\nif [ \"$1\" = -version ]; then printf 'ffmpeg version cli-fixture-v1\\n'; exit 0; fi\nfor output do :; done\nprintf '%s' derived > \"$output\"\n",
    )
}

#[cfg(unix)]
pub(in crate::unit_tests::workflow_command_tests) fn failing_program(root: &Path) -> PathBuf {
    super::executable(
        root,
        "failing.sh",
        "#!/bin/sh\nif [ \"$1\" = -version ]; then printf 'ffmpeg version cli-failing-v1\\n'; exit 0; fi\nexit 9\n",
    )
}

#[cfg(unix)]
pub(in crate::unit_tests::workflow_command_tests) fn fake_ffprobe(
    root: &Path,
    index: u32,
) -> PathBuf {
    let json = format!(
        r#"{{"streams":[{{"index":{index},"codec_type":"audio","codec_name":"pcm_s16le","time_base":"1/48000","start_time":"0.000000","duration":"1.000000","sample_rate":"48000","channels":2,"channel_layout":"stereo"}}],"format":{{"format_name":"wav","duration":"1.000000"}}}}"#
    );
    super::executable(
        root,
        "ffprobe.sh",
        &format!("#!/bin/sh\nif [ \"$1\" = -version ]; then printf 'ffprobe version cli-fixture-v1\\n'; exit 0; fi\nprintf '%s' '{json}'\n"),
    )
}
