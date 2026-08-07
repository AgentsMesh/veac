# Auxiliary And Adaptive Delivery

## DomainTypes

```text
ImageFormat, CaptionSidecarFormat, AudioStemFormat, AudioMixSource, VideoScope
AudioChannelLayout, GifPlayback, GifDither
HlsAudioChoice, HlsRendition, HlsProfile, HlsProfileChoice
```

Every constructor below returns the shared keyed `Deliverable` descriptor and is later included in one
`delivery(...)` aggregate. Target-kind compatibility is part of each operation contract.

## Images And Caption Sidecars

```veac
image_png() -> ImageFormat
image_jpeg() -> ImageFormat
image_tiff() -> ImageFormat
image_exr() -> ImageFormat
caption_srt() -> CaptionSidecarFormat
caption_webvtt() -> CaptionSidecarFormat
caption_ass() -> CaptionSidecarFormat
deliverable_image_frames(key: identifier, target: DeliverableTarget,
                         format: ImageFormat, start_number: int) -> Deliverable
deliverable_caption_sidecar(key: identifier, target: DeliverableTarget,
                            format: CaptionSidecarFormat,
                            layers: list<Layer>) -> Deliverable
```

Image frames require an image-sequence target and non-negative start number. Caption sidecars require a
file target and at least one unique caption Layer owned by the delivery Sequence.

## Stems, Audio Files, And Scopes

```veac
stem_wav() -> AudioStemFormat
stem_flac() -> AudioStemFormat
mix_master() -> AudioMixSource
mix_layer(layer: Layer) -> AudioMixSource
mix_bus(bus: AudioBus) -> AudioMixSource
scope_waveform() -> VideoScope
scope_vectorscope() -> VideoScope
scope_histogram() -> VideoScope
channel_mono() -> AudioChannelLayout
channel_stereo() -> AudioChannelLayout
deliverable_audio_stem(key: identifier, target: DeliverableTarget,
                       format: AudioStemFormat, encoding: AudioOutput,
                       source: AudioMixSource) -> Deliverable
deliverable_mp3(key: identifier, target: DeliverableTarget,
                source: AudioMixSource, bitrate_bps: int, sample_rate_hz: int,
                channels: AudioChannelLayout) -> Deliverable
deliverable_scope(key: identifier, target: DeliverableTarget, scope: VideoScope,
                  at: time, canvas: Canvas, format: ImageFormat) -> Deliverable
```

These artifacts require file targets. Mix Layer must be audio-capable. MP3 bitrate/sample rate are
positive and channel layout is closed mono/stereo. Scope time must lie within the delivery Sequence;
its canvas is positive integer pixels after unit normalization.

## GIF And Still Image

```veac
gif_once() -> GifPlayback
gif_forever() -> GifPlayback
gif_times(count: int) -> GifPlayback
gif_dither_bayer() -> GifDither
gif_dither_floyd_steinberg() -> GifDither
gif_dither_sierra2() -> GifDither
gif_dither_none() -> GifDither
deliverable_gif(key: identifier, target: DeliverableTarget,
                playback: GifPlayback, dither: GifDither) -> Deliverable
deliverable_still(key: identifier, target: DeliverableTarget,
                  containing: time, format: ImageFormat) -> Deliverable
```

GIF count is positive. Still selection means the frame whose half-open presentation interval contains
the requested Sequence time. Both require file targets and raster settings.

## HLS Audio

```veac
hls_audio_none() -> HlsAudioChoice
hls_audio_aac(source: AudioMixSource, bitrate_bps: int, sample_rate_hz: int,
              channels: AudioChannelLayout) -> HlsAudioChoice
```

HLS exposes AAC only in v6. Bitrate and sample rate are positive and shared by every rendition.

## HLS Renditions

```veac
hls_profile_baseline() -> HlsProfile
hls_profile_main() -> HlsProfile
hls_profile_high() -> HlsProfile
hls_profile_auto() -> HlsProfileChoice
hls_profile_present(value: HlsProfile) -> HlsProfileChoice
hls_rendition(key: identifier, canvas: Canvas, target_bps: int, max_bps: int,
              buffer_size_bits: int, profile: HlsProfileChoice,
              level: VideoLevelChoice, color: VideoColorChoice,
              b_frames: BFramePolicy) -> HlsRendition
```

HLS renditions are H.264 capped-bitrate recipes. Canvas dimensions, rates, and buffer are positive;
max is at least target. Level, profile, pixel/color constraints follow the shared H.264 contract.

## Adaptive Package

```veac
deliverable_hls(key: identifier, target: DeliverableTarget, segment_duration: time,
                audio: HlsAudioChoice,
                renditions: list<HlsRendition>) -> Deliverable
```

HLS requires a package target, positive segment duration, and at least one rendition. Rendition keys and
canvas sizes are unique, ordered from lowest to highest pixel area, and use one compatible color policy.
All auxiliary deliverables are immutable descriptors; only `delivery` and `Project.with_delivery` emit
graph topology, as specified in `delivery-video.md`.
