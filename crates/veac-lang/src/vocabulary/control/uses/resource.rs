use super::super::use_macro::define_control_uses;

define_control_uses! {
    LOCATOR_FIELD => "locator" @ ResourceMember : FieldIntroducer;
    STREAMS_FIELD => "streams" @ ResourceMember : FieldIntroducer;
    LOCAL_PATH_FIELD => "path" @ LocalResourceLocatorMember : FieldIntroducer;
    REMOTE_URI_FIELD => "uri" @ RemoteResourceLocatorMember : FieldIntroducer;
    REMOTE_IDENTITY_FIELD => "identity" @ RemoteResourceLocatorMember : FieldIntroducer;
    STREAMS_VIDEO_FIELD => "video" @ ResourceStreamsMember : FieldIntroducer;
    STREAMS_AUDIO_FIELD => "audio" @ ResourceStreamsMember : FieldIntroducer;
}
