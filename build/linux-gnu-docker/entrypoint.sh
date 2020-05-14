#!/usr/bin/env bash

set -eux -o pipefail

# Get noaa-apt version from Cargo.toml
NOAA_APT_VERSION=$(awk '/^version =/{print substr($NF, 2, length($NF)-2)}' Cargo.toml)

PACKAGES_FOLDER=/home/rustacean/src/target/docker_builds

X86_64_GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-linux-gnu"
X86_64_NOGUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-linux-gnu-nogui"
X86_64_GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$X86_64_GUI_PACKAGE_NAME"
X86_64_NOGUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$X86_64_NOGUI_PACKAGE_NAME"

ARMV7_GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-armv7-linux-gnueabihf"
ARMV7_NOGUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-armv7-linux-gnueabihf-nogui"
ARMV7_GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$ARMV7_GUI_PACKAGE_NAME"
ARMV7_NOGUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$ARMV7_NOGUI_PACKAGE_NAME"

export PKG_CONFIG_ALLOW_CROSS=1

# Export these variables, because they are also used on debian/rules
export CARGO_BINARY=/home/rustacean/.cargo/bin/cargo

# Build with GUI

"$CARGO_BINARY" build --target=x86_64-unknown-linux-gnu --release --features static_ssl

rm -r "$X86_64_GUI_PACKAGE_FOLDER" || true
mkdir -p "$X86_64_GUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$X86_64_GUI_PACKAGE_FOLDER/"
cp -r "./test" "$X86_64_GUI_PACKAGE_FOLDER/"
cp -r "./res" "$X86_64_GUI_PACKAGE_FOLDER/"
rm -r "$X86_64_GUI_PACKAGE_FOLDER/test/results" || true

# Build without GUI

"$CARGO_BINARY" build --target=x86_64-unknown-linux-gnu --release --no-default-features --features static_ssl

rm -r "$X86_64_NOGUI_PACKAGE_FOLDER" || true
mkdir -p "$X86_64_NOGUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$X86_64_NOGUI_PACKAGE_FOLDER/"
cp -r "./test" "$X86_64_NOGUI_PACKAGE_FOLDER/"
cp -r "./res" "$X86_64_NOGUI_PACKAGE_FOLDER/"
rm -r "$X86_64_NOGUI_PACKAGE_FOLDER/test/results" || true

# Build with GUI for Raspberry Pi

# Otherwise it can't find `arm-linux-gnueabihf-gcc` because of the missing `-6`
export TARGET_CC=arm-linux-gnueabihf-gcc-6

# Otherwise for some reason it can't find
# `/usr/lib/arm-linux-gnueabihf/pkgconfig/glib-2.0.pc`
export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig/

"$CARGO_BINARY" build --target=armv7-unknown-linux-gnueabihf --release --features static_ssl

rm -r "$ARMV7_GUI_PACKAGE_FOLDER" || true
mkdir -p "$ARMV7_GUI_PACKAGE_FOLDER"
cp ./target/armv7-unknown-linux-gnueabihf/release/noaa-apt "$ARMV7_GUI_PACKAGE_FOLDER/"
cp -r "./test" "$ARMV7_GUI_PACKAGE_FOLDER/"
cp -r "./res" "$ARMV7_GUI_PACKAGE_FOLDER/"
rm -r "$ARMV7_GUI_PACKAGE_FOLDER/test/results" || true

# Build without GUI for Raspberry Pi

"$CARGO_BINARY" build --target=armv7-unknown-linux-gnueabihf --release --no-default-features --features static_ssl

rm -r "$ARMV7_NOGUI_PACKAGE_FOLDER" || true
mkdir -p "$ARMV7_NOGUI_PACKAGE_FOLDER"
cp ./target/armv7-unknown-linux-gnueabihf/release/noaa-apt "$ARMV7_NOGUI_PACKAGE_FOLDER/"
cp -r "./test" "$ARMV7_NOGUI_PACKAGE_FOLDER/"
cp -r "./res" "$ARMV7_NOGUI_PACKAGE_FOLDER/"
rm -r "$ARMV7_NOGUI_PACKAGE_FOLDER/test/results" || true

# Build deb

# Verbose build for dpkg-buildpackage
DH_VERBOSE=1

# Indicate resources folder before compiling
export NOAA_APT_RES_DIR="/usr/share/noaa-apt"

# -us -uc: Do not sign anything. When upgrading to a newer Debian version I
#          should change to --no-sign
# -d: Do not check build dependencies, because cargo is a build dependency and I
#     install it manually using rustup. Also i'm statically linking libssl so I
#     don't need libssl-dev
# -b: Binary package only
dpkg-buildpackage -us -uc -d -b

mv ../noaa-apt*.deb "$PACKAGES_FOLDER/"

# Zip everything

pushd "$X86_64_GUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$X86_64_GUI_PACKAGE_NAME.zip" ./*
popd

pushd "$X86_64_NOGUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$X86_64_NOGUI_PACKAGE_NAME.zip" ./*
popd

pushd "$ARMV7_GUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$ARMV7_GUI_PACKAGE_NAME.zip" ./*
popd

pushd "$ARMV7_NOGUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$ARMV7_NOGUI_PACKAGE_NAME.zip" ./*
popd
