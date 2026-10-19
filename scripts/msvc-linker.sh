#!/bin/bash
exec /home/ursa/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld -flavor link "$@"
