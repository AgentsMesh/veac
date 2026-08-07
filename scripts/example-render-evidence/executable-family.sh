#!/usr/bin/env bash

for executable_module in \
  executable-family-common executable-programming-language executable-audio-caption \
  executable-centered-dissolve executable-local-image executable-text-family; do
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/example-render-evidence/$executable_module.sh"
done
unset executable_module

check_executable_family_evidence() {
  check_programming_language_evidence
  check_executable_audio_caption_evidence
  check_executable_centered_dissolve_evidence
  check_executable_local_image_evidence
  check_executable_text_family_evidence
}
