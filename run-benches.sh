#!/bin/bash

set -x;

# CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind" \
cargo +nightly bench \
  --features minivec_nightly
