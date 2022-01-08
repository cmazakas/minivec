#!/bin/bash -e

set -x;

cargo +nightly clippy \
  && cargo +nightly build \
  && CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind" \
     CARGO_BUILD_RUSTFLAGS="-C target-feature=+avx" \
     cargo +nightly test $@ \
  && MIRIFLAGS="-Zmiri-tag-raw-pointers -Zmiri-symbolic-alignment-check" \
     cargo +nightly miri test $@
  && cargo +nightly build --features minivec_nightly \
  && CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind" \
     CARGO_BUILD_RUSTFLAGS="-C target-feature=+avx" \
     cargo +nightly test --features minivec_nightly $@ \
  && MIRIFLAGS="-Zmiri-tag-raw-pointers -Zmiri-symbolic-alignment-check" \
     cargo +nightly miri test --features minivec_nightly $@
