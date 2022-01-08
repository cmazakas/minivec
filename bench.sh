#!/bin/bash

# CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind" \
RUSTFLAGS="-C target-feature=+avx" \
cargo +nightly bench \
  --bench minivec \
  --features minivec_nightly
