#!/bin/sh

set -e
set -x

PACKAGE_FOLDER=/src/target/x86_64-unknown-linux-gnu/package/

/home/rustacean/.cargo/bin/cargo build --target=x86_64-unknown-linux-gnu --release
