#!/bin/bash

RUSTFLAGS="-Z instrument-coverage" LLVM_PROFILE_FILE="minivec-%m.profraw" \
cargo test --tests \
  && cargo profdata -- merge -sparse minivec-*.profraw -o minivec.profdata \
  && cargo cov -- report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-Z instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --use-color \
  --ignore-filename-regex='/.cargo/registry' \
  --instr-profile=minivec.profdata \
  --summary-only \
  && cargo cov -- show \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-Z instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --use-color \
  --ignore-filename-regex='/.cargo/registry' \
  --instr-profile=minivec.profdata \
  --show-instantiations \
  --show-line-counts-or-regions \
  --Xdemangler=rustfilt \
  --format=html > minivec-coverage.html
