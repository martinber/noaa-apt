#!/bin/sh

set -e
set -x

GUI_PACKAGE_FOLDER=/home/rustacean/src/target/x86_64-unknown-linux-gnu/package/
NOGUI_PACKAGE_FOLDER=/home/rustacean/src/target/x86_64-unknown-linux-gnu/no-gui-package/
PKG_CONFIG_ALLOW_CROSS=1
OPENSSL_DIR=/usr/local
OPENSSL_LIB_DIR=/usr/local/lib/
OPENSSL_INCLUDE_DIR=/usr/local/include
OPENSSL_STATIC=yes

# Build with GUI

/home/rustacean/.cargo/bin/cargo build --target=x86_64-unknown-linux-gnu --release

rm -r "$GUI_PACKAGE_FOLDER" || true
mkdir -p "$GUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$GUI_PACKAGE_FOLDER/"
cp -r "./test" "$GUI_PACKAGE_FOLDER/"

# Build without GUI

/home/rustacean/.cargo/bin/cargo build --target=x86_64-unknown-linux-gnu --release --no-default-features

rm -r "$NOGUI_PACKAGE_FOLDER" || true
mkdir -p "$NOGUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$NOGUI_PACKAGE_FOLDER/"
cp -r "./test" "$NOGUI_PACKAGE_FOLDER/"
