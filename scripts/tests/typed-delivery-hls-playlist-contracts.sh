#!/usr/bin/env bash

run_delivery_hls_playlist_contracts() {
  local root stream segment

  root="$TMP_DIR/wrong-hls-codecs-attribute"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's/CODECS="avc1.64001e,mp4a.40.2"/CODECS="avc1.64001e,mp4a.40.2,mp4a.40.5"/' \
    "$stream/master.m3u8" >"$stream/master.tmp"
  mv "$stream/master.tmp" "$stream/master.m3u8"
  expect_typed_failure wrong_hls_codecs "$root" check_delivery_hls \
    "attributes do not match its codecs and raster"

  root="$TMP_DIR/wrong-hls-bandwidth-attribute"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's/BANDWIDTH=1200000/BANDWIDTH=999/' "$stream/master.m3u8" >"$stream/master.tmp"
  mv "$stream/master.tmp" "$stream/master.m3u8"
  expect_typed_failure wrong_hls_bandwidth "$root" check_delivery_hls \
    "HLS BANDWIDTH"

  root="$TMP_DIR/fake-hls-independent-tag"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's/^#EXT-X-INDEPENDENT-SEGMENTS$/#EXT-X-INDEPENDENT-SEGMENTS-FAKE/' \
    "$stream/rendition-rnd_mobile.m3u8" >"$stream/rendition.tmp"
  mv "$stream/rendition.tmp" "$stream/rendition-rnd_mobile.m3u8"
  expect_typed_failure fake_hls_independent "$root" check_delivery_hls \
    "exactly one #EXT-X-INDEPENDENT-SEGMENTS tag"

  root="$TMP_DIR/duplicate-hls-independent-tag"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed '/^#EXT-X-INDEPENDENT-SEGMENTS$/p' "$stream/rendition-rnd_mobile.m3u8" >"$stream/rendition.tmp"
  mv "$stream/rendition.tmp" "$stream/rendition-rnd_mobile.m3u8"
  expect_typed_failure duplicate_hls_independent "$root" check_delivery_hls \
    "exactly one #EXT-X-INDEPENDENT-SEGMENTS tag"

  root="$TMP_DIR/unreferenced-hls-segment"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  segment=$(find "$stream" -maxdepth 1 -type f -name 'segment-rnd_mobile-*.ts' |
    LC_ALL=C sort | sed -n '1p')
  cp "$segment" "$stream/segment-orphan-000000.ts"
  expect_typed_failure unreferenced_hls_segment "$root" check_delivery_hls \
    "missing or unreachable members"
}
