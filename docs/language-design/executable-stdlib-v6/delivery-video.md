# Video Delivery

## DomainTypes

```text
Delivery, Deliverable, DeliverableTarget, RasterChoice, CaptionOutput
VideoDelivery, VideoOutput, OutputContainer, VideoCodec, PixelFormat, AlphaMode
VideoRateControl, VideoProfile, VideoProfileChoice, VideoLevel, VideoLevelChoice
VideoColorChoice, GopPolicy, BFramePolicy, PassMode, HardwareSelection
AudioOutput, AudioCodec, EmbeddedAudioChoice
```

## Delivery Root And Target

```veac
caption_burn_in() -> CaptionOutput
caption_discard() -> CaptionOutput
raster_none() -> RasterChoice
raster_settings(canvas: Canvas, frame_rate: FrameRate,
                captions: CaptionOutput) -> RasterChoice
delivery_file(name: text) -> DeliverableTarget
delivery_image_sequence(pattern: text) -> DeliverableTarget
delivery_package(name: text) -> DeliverableTarget
```

Targets are normalized artifact-relative names, never arbitrary backend paths. Raster absence is legal
only when every deliverable is non-raster.

## Container, Codec, Pixel, And Alpha

```veac
container_mp4() -> OutputContainer
container_mov() -> OutputContainer
container_mkv() -> OutputContainer
container_webm() -> OutputContainer
container_mxf() -> OutputContainer
video_h264() -> VideoCodec
video_h265() -> VideoCodec
video_vp9() -> VideoCodec
video_av1() -> VideoCodec
video_prores() -> VideoCodec
video_dnxhr() -> VideoCodec
pixel_yuv420p() -> PixelFormat
pixel_yuv420p10le() -> PixelFormat
pixel_yuv422p() -> PixelFormat
pixel_yuv422p10le() -> PixelFormat
pixel_yuv444p10le() -> PixelFormat
pixel_yuva444p10le() -> PixelFormat
alpha_opaque() -> AlphaMode
alpha_straight() -> AlphaMode
```

The verifier enforces codec/container, pixel/profile, and alpha compatibility. Straight alpha is the
ProRes 4444/YUVA contract; no backend flag can bypass it.

## Rate And Codec Tuning

```veac
video_crf(value: int) -> VideoRateControl
video_bitrate(target_bps: int) -> VideoRateControl
video_capped_bitrate(target_bps: int, max_bps: int,
                     buffer_size_bits: int) -> VideoRateControl
video_lossless() -> VideoRateControl
gop_auto() -> GopPolicy
gop_frames(value: int) -> GopPolicy
b_frames_auto() -> BFramePolicy
b_frames_count(value: int) -> BFramePolicy
```

CRF range is codec-specific: H.264/H.265 `[0,51]`, VP9/AV1 `[0,63]`, and unavailable for ProRes/DNxHR.
Bitrates are positive and bounded by canonical limits; capped max is at least target. GOP is positive;
B-frame count is `[0,16]` and H.264 baseline requires zero or auto.

## Profile And Level

```veac
profile_h264_baseline() -> VideoProfile
profile_h264_main() -> VideoProfile
profile_h264_high() -> VideoProfile
profile_h264_high10() -> VideoProfile
profile_h265_main() -> VideoProfile
profile_h265_main10() -> VideoProfile
profile_vp9_0() -> VideoProfile
profile_vp9_2() -> VideoProfile
profile_av1_main() -> VideoProfile
profile_prores_4444() -> VideoProfile
profile_dnxhr_lb() -> VideoProfile
profile_dnxhr_sq() -> VideoProfile
profile_dnxhr_hq() -> VideoProfile
profile_dnxhr_hqx() -> VideoProfile
profile_dnxhr_444() -> VideoProfile
video_profile_auto() -> VideoProfileChoice
video_profile_present(value: VideoProfile) -> VideoProfileChoice
h264_level(major: int, minor: int) -> VideoLevel
h265_level(major: int, minor: int) -> VideoLevel
vp9_level(major: int, minor: int) -> VideoLevel
av1_level(major: int, minor: int) -> VideoLevel
video_level_auto() -> VideoLevelChoice
video_level_present(value: VideoLevel) -> VideoLevelChoice
```

Each level constructor validates its `(major, minor)` pair against the canonical codec table and lowers
to its decimal spelling. ProRes and DNxHR have no level variant. This keeps levels closed without
exposing the canonical string field.

## Color, Pass, And Hardware

```veac
video_color_unspecified() -> VideoColorChoice
video_color_present(value: ColorSpace) -> VideoColorChoice
pass_single() -> PassMode
pass_two() -> PassMode
hardware_auto() -> HardwareSelection
hardware_software() -> HardwareSelection
hardware_videotoolbox() -> HardwareSelection
hardware_nvenc() -> HardwareSelection
hardware_qsv() -> HardwareSelection
hardware_vaapi() -> HardwareSelection
```

Hardware selection is an execution preference with closed semantics, not a backend option map. Two-pass
is valid only for bitrate control and not with hardware encoding.

## Embedded Audio

```veac
audio_aac() -> AudioCodec
audio_opus() -> AudioCodec
audio_flac() -> AudioCodec
audio_pcm_s16le() -> AudioCodec
audio_pcm_s24le() -> AudioCodec
audio_pcm_s32le() -> AudioCodec
audio_output(codec: AudioCodec, sample_rate_hz: int, channels: int) -> AudioOutput
embedded_audio_none() -> EmbeddedAudioChoice
embedded_audio_present(value: AudioOutput) -> EmbeddedAudioChoice
```

Sample rate and channels are positive and codec/container compatible.

## Video Artifact And Graph Operations

```veac
video_output(codec: VideoCodec, pixel: PixelFormat, alpha: AlphaMode,
             color: VideoColorChoice, rate: VideoRateControl, gop: GopPolicy,
             b_frames: BFramePolicy, profile: VideoProfileChoice,
             level: VideoLevelChoice) -> VideoOutput
video_delivery(container: OutputContainer, video: VideoOutput, audio: EmbeddedAudioChoice,
               optimize_for_streaming: bool, passes: PassMode,
               hardware: HardwareSelection) -> VideoDelivery
deliverable_video(key: identifier, target: DeliverableTarget,
                  settings: VideoDelivery) -> Deliverable
delivery(key: identifier, sequence: Sequence, raster: RasterChoice,
         artifacts: list<Deliverable>) -> Delivery
Project.with_delivery(delivery: Delivery) -> Project
```

`delivery` and `Project.with_delivery` are `GraphEmit`; deliverable constructors are Pure keyed
descriptors. Artifact keys are unique and the list is non-empty. Video requires a file target and
raster settings. Optimization, pass, hardware, and codec choices are validated as one recipe before
attachment.
