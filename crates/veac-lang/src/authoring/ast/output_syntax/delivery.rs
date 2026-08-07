use super::super::{
    AdaptivePackageFormat, AnimatedImageFormat, AudioFileFormat, AudioMixSourceKind, HlsAudioCodec,
    HlsH264ProfileDecl, HlsVideoCodec,
};

crate::impl_local_syntax_tokens!(AudioMixSourceKind,
    AudioMixSourceKind::Master => "master", AudioMixSourceKind::Track => "track",
    AudioMixSourceKind::Bus => "bus",
);
crate::impl_local_syntax_tokens!(AudioFileFormat,
    AudioFileFormat::Mp3 => "mp3",
);
crate::impl_local_syntax_tokens!(AnimatedImageFormat,
    AnimatedImageFormat::Gif => "gif",
);
crate::impl_local_syntax_tokens!(AdaptivePackageFormat,
    AdaptivePackageFormat::Hls => "hls",
);
crate::impl_local_syntax_tokens!(HlsVideoCodec,
    HlsVideoCodec::H264 => "h264",
);
crate::impl_local_syntax_tokens!(HlsAudioCodec,
    HlsAudioCodec::Aac => "aac",
);
crate::impl_local_syntax_tokens!(HlsH264ProfileDecl,
    HlsH264ProfileDecl::Baseline => "baseline", HlsH264ProfileDecl::Main => "main",
    HlsH264ProfileDecl::High => "high",
);
