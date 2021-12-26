#!/bin/bash -e

cargo +nightly clippy \
  && cargo +nightly build \
  && CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --error-exitcode=255 --suppressions=suppressions.log" \
     CARGO_BUILD_RUSTFLAGS="-C target-feature=+avx" \
     cargo +nightly test $@ \
  && MIRIFLAGS="-Zmiri-tag-raw-pointers -Zmiri-symbolic-alignment-check" \
     cargo +nightly miri test $@
