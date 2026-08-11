#!/usr/bin/env bash

readonly COVERAGE_MINIMUM="95.03"
# Keep one path-boundary prefix so llvm-cov can compose this with package filters.
readonly COVERAGE_TEST_SOURCE_REGEX='(^|/)((tests?|unit_tests|test_support)(/|[.]rs$)|(test_[^/]*|[^/]*_(test|tests|test_support))[.]rs$)'

is_coverage_test_source() {
  local path="/${1#./}"
  [[ "$path" =~ $COVERAGE_TEST_SOURCE_REGEX ]]
}
