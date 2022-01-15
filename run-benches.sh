#!/bin/bash

set -x;

# CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind" \
RUSTFLAGS="-C target-feature=+sse2,+avx,+avx2" \
cargo +nightly bench \
  --features minivec_nightly \
  --bench minivec \
  -- --noplot
