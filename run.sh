#!/bin/bash
cargo clippy && cargo build && CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind" cargo test
