use std::ffi::OsString;

const SAFE_PROTOCOLS: &str = "file,pipe";
const SAFE_FORMATS: &str = concat!(
    "aac,ac3,aiff,amr,ape,apng,asf,au,av1,avi,bmp_pipe,caf,dds_pipe,",
    "dpx_pipe,dts,dv,eac3,exr_pipe,flac,flv,gif,gxf,h264,hevc,ico,ivf,",
    "jpeg_pipe,jpegls_pipe,jpegxl_anim,jpegxl_pipe,m4v,matroska,mjpeg,",
    "mjpeg_2000,mlp,mov,mp3,mpeg,mpegts,mpegtsraw,mxf,nut,obu,ogg,oma,",
    "png_pipe,psd_pipe,rawvideo,rm,sox,swf,tiff_pipe,truehd,tta,vvc,w64,",
    "wav,webp_pipe,wtv,wv,yuv4mpegpipe",
);

pub(crate) fn string_arguments() -> [String; 4] {
    arguments().map(str::to_owned)
}

pub(crate) fn os_arguments() -> [OsString; 4] {
    arguments().map(OsString::from)
}

fn arguments() -> [&'static str; 4] {
    [
        "-protocol_whitelist",
        SAFE_PROTOCOLS,
        "-format_whitelist",
        SAFE_FORMATS,
    ]
}

#[cfg(test)]
#[path = "input_policy/tests.rs"]
mod tests;
