#!/usr/bin/env bash

set -eux -o pipefail

# Get noaa-apt version from Cargo.toml
NOAA_APT_VERSION=$(awk '/^version =/{print substr($NF, 2, length($NF)-2)}' Cargo.toml)

PACKAGES_FOLDER=/home/rustacean/src/target/docker_builds
GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-linux-gnu"
NOGUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-linux-gnu-nogui"
GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$GUI_PACKAGE_NAME"
NOGUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$NOGUI_PACKAGE_NAME"

PKG_CONFIG_ALLOW_CROSS=1


# These environment variables are used by cargo when compiling rust-openssl
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
rm -r "$GUI_PACKAGE_FOLDER/test/results" || true

# Build without GUI

/home/rustacean/.cargo/bin/cargo build --target=x86_64-unknown-linux-gnu --release --no-default-features

rm -r "$NOGUI_PACKAGE_FOLDER" || true
mkdir -p "$NOGUI_PACKAGE_FOLDER"
cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$NOGUI_PACKAGE_FOLDER/"
cp -r "./test" "$NOGUI_PACKAGE_FOLDER/"
rm -r "$NOGUI_PACKAGE_FOLDER/test/results" || true

# Build deb

# Verbose build for dpkg-buildpackage
DH_VERBOSE=1

# -us -uc: Do not sign anything. When upgrading to a newer Debian version I
#          should change to --no-sign
# -d: Do not check build dependencies, because cargo is a build dependency and I
#     install it manually using rustup. Also i'm statically linking libssl so I
#     don't need libssl-dev
# -b: Binary package only
#
# Also set location if cargo, used by debian/rules
env CARGO_BINARY=/home/rustacean/.cargo/bin/cargo dpkg-buildpackage -us -uc -d -b

mv ../noaa-apt*.deb "$PACKAGES_FOLDER/"

# Zip GUI and NOGUI folders

pushd "$GUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$GUI_PACKAGE_NAME.zip" ./*
popd

pushd "$NOGUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$NOGUI_PACKAGE_NAME.zip" ./*
popd
