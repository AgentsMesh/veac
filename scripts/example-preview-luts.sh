#!/usr/bin/env bash

write_preview_luts() {
  local fixtures=$1

  if [[ ! -f "$fixtures/lut1d.cube" ]]; then
    printf '%s\n' \
      'TITLE "VEAC Preview 1D"' \
      'LUT_1D_SIZE 4' \
      'DOMAIN_MIN 0.0 0.0 0.0' \
      'DOMAIN_MAX 1.0 1.0 1.0' \
      '0.000 0.000 0.000' \
      '0.300 0.350 0.400' \
      '0.700 0.680 0.620' \
      '1.000 0.950 0.850' > "$fixtures/lut1d.cube"
  fi

  if [[ ! -f "$fixtures/lut3d.cube" ]]; then
    printf '%s\n' \
      'TITLE "VEAC Preview 3D"' \
      'LUT_3D_SIZE 2' \
      'DOMAIN_MIN 0.0 0.0 0.0' \
      'DOMAIN_MAX 1.0 1.0 1.0' \
      '0.000 0.000 0.000' \
      '0.000 0.000 0.900' \
      '0.000 0.900 0.000' \
      '0.000 0.900 0.900' \
      '0.900 0.000 0.000' \
      '0.900 0.000 0.900' \
      '0.900 0.900 0.000' \
      '1.000 0.950 0.850' > "$fixtures/lut3d.cube"
  fi
}
