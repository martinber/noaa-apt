#!/usr/bin/env bash

set -eux -o pipefail

# Get noaa-apt version from Cargo.toml
NOAA_APT_VERSION=$(awk '/^version =/{print substr($NF, 2, length($NF)-2)}' Cargo.toml)

PACKAGES_FOLDER=/home/rustacean/src/target/docker_builds

LINUX_GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-linux-gnu"
LINUX_NOGUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-linux-gnu-nogui"
LINUX_GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$LINUX_GUI_PACKAGE_NAME"
LINUX_NOGUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$LINUX_NOGUI_PACKAGE_NAME"

RPI_GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-armv7-linux-gnueabihf"
RPI_NOGUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-armv7-linux-gnueabihf-nogui"
RPI_GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$RPI_GUI_PACKAGE_NAME"
RPI_NOGUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$RPI_NOGUI_PACKAGE_NAME"

export PKG_CONFIG_ALLOW_CROSS=1

# Export these variables, because they are also used on debian/rules
export CARGO_BINARY=/home/rustacean/.cargo/bin/cargo

# Used by cargo when compiling rust-openssl
# Also inside of debian/rules
export OPENSSL_DIR=/usr/local/openssl
export OPENSSL_LIB_DIR=/usr/local/openssl/lib/
export OPENSSL_INCLUDE_DIR=/usr/local/openssl/include
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_DIR=/usr/local/openssl_armv7
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/usr/local/openssl_armv7/lib/
export ARMV7_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/usr/local/openssl_armv7/include
export OPENSSL_STATIC=yes

# Build without GUI for Raspberry Pi

export TARGET_CC=arm-linux-gnueabihf-gcc-6

"$CARGO_BINARY" build --target=armv7-unknown-linux-gnueabihf --release --no-default-features

rm -r "$RPI_NOGUI_PACKAGE_FOLDER" || true
mkdir -p "$RPI_NOGUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$RPI_NOGUI_PACKAGE_FOLDER/"
cp -r "./test" "$RPI_NOGUI_PACKAGE_FOLDER/"
rm -r "$RPI_NOGUI_PACKAGE_FOLDER/test/results" || true

# Build with GUI for Raspberry Pi

"$CARGO_BINARY" build --target=armv7-unknown-linux-gnueabihf --release

rm -r "$RPI_GUI_PACKAGE_FOLDER" || true
mkdir -p "$RPI_GUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$RPI_GUI_PACKAGE_FOLDER/"
cp -r "./test" "$RPI_GUI_PACKAGE_FOLDER/"
rm -r "$RPI_GUI_PACKAGE_FOLDER/test/results" || true

# Build with GUI

"$CARGO_BINARY" build --target=x86_64-unknown-linux-gnu --release

rm -r "$LINUX_GUI_PACKAGE_FOLDER" || true
mkdir -p "$LINUX_GUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$LINUX_GUI_PACKAGE_FOLDER/"
cp -r "./test" "$LINUX_GUI_PACKAGE_FOLDER/"
rm -r "$LINUX_GUI_PACKAGE_FOLDER/test/results" || true

# Build without GUI

"$CARGO_BINARY" build --target=x86_64-unknown-linux-gnu --release --no-default-features

rm -r "$LINUX_NOGUI_PACKAGE_FOLDER" || true
mkdir -p "$LINUX_NOGUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$LINUX_NOGUI_PACKAGE_FOLDER/"
cp -r "./test" "$LINUX_NOGUI_PACKAGE_FOLDER/"
rm -r "$LINUX_NOGUI_PACKAGE_FOLDER/test/results" || true

# Build deb

# Verbose build for dpkg-buildpackage
DH_VERBOSE=1

# -us -uc: Do not sign anything. When upgrading to a newer Debian version I
#          should change to --no-sign
# -d: Do not check build dependencies, because cargo is a build dependency and I
#     install it manually using rustup. Also i'm statically linking libssl so I
#     don't need libssl-dev
# -b: Binary package only
dpkg-buildpackage -us -uc -d -b

mv ../noaa-apt*.deb "$PACKAGES_FOLDER/"

# Zip GUI and NOGUI folders

pushd "$LINUX_GUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$LINUX_GUI_PACKAGE_NAME.zip" ./*
popd

pushd "$LINUX_NOGUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$LINUX_NOGUI_PACKAGE_NAME.zip" ./*
popd
