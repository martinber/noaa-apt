#!/usr/bin/env bash

set -eux -o pipefail

# If the argument $1 is unset, it will build ALL. Possible values:
# ALL, X86_64_GUI, X86_64_GUI_DEB, X86_64_NOGUI, ARMV7_GUI, ARMV7_NOGUI, AARCH64_GUI, AARCH64_NOGUI
RELEASE="${1:-ALL}"
echo "Building $RELEASE"

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

AARCH64_GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-aarch64-linux-gnu"
AARCH64_NOGUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-aarch64-linux-gnu-nogui"
AARCH64_GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$AARCH64_GUI_PACKAGE_NAME"
AARCH64_NOGUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$AARCH64_NOGUI_PACKAGE_NAME"

export PKG_CONFIG_ALLOW_CROSS=1

# Export these variables, because they are also used on debian/rules
export CARGO_BINARY=/home/rustacean/.cargo/bin/cargo

# Build with GUI
# --------------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "X86_64_GUI" ]]; then

    unset NOAA_APT_RES_DIR
    "$CARGO_BINARY" build --target=x86_64-unknown-linux-gnu --release --features static_ssl

    rm -r "$X86_64_GUI_PACKAGE_FOLDER" || true
    mkdir -p "$X86_64_GUI_PACKAGE_FOLDER"
    cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$X86_64_GUI_PACKAGE_FOLDER/"
    cp -r "./test" "$X86_64_GUI_PACKAGE_FOLDER/"
    cp -r "./res" "$X86_64_GUI_PACKAGE_FOLDER/"
    cp -r "./build/run-noaa-apt.sh" "$X86_64_GUI_PACKAGE_FOLDER/"
    rm -r "$X86_64_GUI_PACKAGE_FOLDER/test/results" || true

    pushd "$X86_64_GUI_PACKAGE_FOLDER/"
    zip -rq "$PACKAGES_FOLDER/$X86_64_GUI_PACKAGE_NAME.zip" ./*
    popd

fi

# Build without GUI
# -----------------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "X86_64_NOGUI" ]]; then

    unset NOAA_APT_RES_DIR
    "$CARGO_BINARY" build --target=x86_64-unknown-linux-gnu --release --no-default-features --features static_ssl

    rm -r "$X86_64_NOGUI_PACKAGE_FOLDER" || true
    mkdir -p "$X86_64_NOGUI_PACKAGE_FOLDER"
    cp ./target/x86_64-unknown-linux-gnu/release/noaa-apt "$X86_64_NOGUI_PACKAGE_FOLDER/"
    cp -r "./test" "$X86_64_NOGUI_PACKAGE_FOLDER/"
    cp -r "./res" "$X86_64_NOGUI_PACKAGE_FOLDER/"
    cp -r "./build/run-noaa-apt.sh" "$X86_64_NOGUI_PACKAGE_FOLDER/"
    rm -r "$X86_64_NOGUI_PACKAGE_FOLDER/test/results" || true

    pushd "$X86_64_NOGUI_PACKAGE_FOLDER/"
    zip -rq "$PACKAGES_FOLDER/$X86_64_NOGUI_PACKAGE_NAME.zip" ./*
    popd

fi

# Build deb
# ---------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "X86_64_GUI_DEB" ]]; then

    DH_VERBOSE=1 # Verbose build for dpkg-buildpackage
    export NOAA_APT_RES_DIR="/usr/share/noaa-apt" # Indicate resources folder before compiling

    # -us -uc: Do not sign anything. When upgrading to a newer Debian version I
    #          should change to --no-sign
    # -d: Do not check build dependencies, because cargo is a build dependency and I
    #     install it manually using rustup. Also i'm statically linking libssl so I
    #     don't need libssl-dev
    # -b: Binary package only
    dpkg-buildpackage -us -uc -d -b

    mv ../noaa-apt*.deb "$PACKAGES_FOLDER/"

fi

# Build with GUI for armv7
# ------------------------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "ARMV7_GUI" ]]; then

    unset NOAA_APT_RES_DIR
    export TARGET_CC=arm-linux-gnueabihf-gcc-8
    export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig/
    "$CARGO_BINARY" build --target=armv7-unknown-linux-gnueabihf --release --features static_ssl

    rm -r "$ARMV7_GUI_PACKAGE_FOLDER" || true
    mkdir -p "$ARMV7_GUI_PACKAGE_FOLDER"
    cp ./target/armv7-unknown-linux-gnueabihf/release/noaa-apt "$ARMV7_GUI_PACKAGE_FOLDER/"
    cp -r "./test" "$ARMV7_GUI_PACKAGE_FOLDER/"
    cp -r "./res" "$ARMV7_GUI_PACKAGE_FOLDER/"
    cp -r "./build/run-noaa-apt.sh" "$ARMV7_GUI_PACKAGE_FOLDER/"
    rm -r "$ARMV7_GUI_PACKAGE_FOLDER/test/results" || true

    pushd "$ARMV7_GUI_PACKAGE_FOLDER/"
    zip -rq "$PACKAGES_FOLDER/$ARMV7_GUI_PACKAGE_NAME.zip" ./*
    popd

fi

# Build without GUI for armv7
# ---------------------------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "ARMV7_NOGUI" ]]; then

    unset NOAA_APT_RES_DIR
    export TARGET_CC=arm-linux-gnueabihf-gcc-8
    export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig/
    "$CARGO_BINARY" build --target=armv7-unknown-linux-gnueabihf --release --no-default-features --features static_ssl

    rm -r "$ARMV7_NOGUI_PACKAGE_FOLDER" || true
    mkdir -p "$ARMV7_NOGUI_PACKAGE_FOLDER"
    cp ./target/armv7-unknown-linux-gnueabihf/release/noaa-apt "$ARMV7_NOGUI_PACKAGE_FOLDER/"
    cp -r "./test" "$ARMV7_NOGUI_PACKAGE_FOLDER/"
    cp -r "./res" "$ARMV7_NOGUI_PACKAGE_FOLDER/"
    rm -r "$ARMV7_NOGUI_PACKAGE_FOLDER/test/results" || true

    pushd "$ARMV7_NOGUI_PACKAGE_FOLDER/"
    zip -rq "$PACKAGES_FOLDER/$ARMV7_NOGUI_PACKAGE_NAME.zip" ./*
    popd

fi

# Build with GUI for arm64
# ------------------------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "AARCH64_GUI" ]]; then

    unset NOAA_APT_RES_DIR
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc-8
    export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc-ar-8
    export AR=aarch64-linux-gnu-gcc-ar-8
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc-8
    export CC=aarch64-linux-gnu-gcc-8
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++-8
    export CXX=aarch64-linux-gnu-g++-8
    export TARGET_CC=aarch64-linux-gnu-gcc-8
    export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig/
    "$CARGO_BINARY" build --target=aarch64-unknown-linux-gnu --release --features static_ssl

    rm -r "$AARCH64_GUI_PACKAGE_FOLDER" || true
    mkdir -p "$AARCH64_GUI_PACKAGE_FOLDER"
    cp ./target/aarch64-unknown-linux-gnu/release/noaa-apt "$AARCH64_GUI_PACKAGE_FOLDER/"
    cp -r "./test" "$AARCH64_GUI_PACKAGE_FOLDER/"
    cp -r "./res" "$AARCH64_GUI_PACKAGE_FOLDER/"
    cp -r "./build/run-noaa-apt.sh" "$AARCH64_GUI_PACKAGE_FOLDER/"
    rm -r "$AARCH64_GUI_PACKAGE_FOLDER/test/results" || true

    pushd "$AARCH64_GUI_PACKAGE_FOLDER/"
    zip -rq "$PACKAGES_FOLDER/$AARCH64_GUI_PACKAGE_NAME.zip" ./*
    popd

fi

# Build without GUI for arm64
# ---------------------------

if [[ "$RELEASE" == "ALL" || "$RELEASE" == "AARCH64_NOGUI" ]]; then

    unset NOAA_APT_RES_DIR
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc-8
    export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc-ar-8
    export AR=aarch64-linux-gnu-gcc-ar-8
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc-8
    export CC=aarch64-linux-gnu-gcc-8
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++-8
    export CXX=aarch64-linux-gnu-g++-8
    export TARGET_CC=aarch64-linux-gnu-gcc-8
    export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig/
    "$CARGO_BINARY" build --target=aarch64-unknown-linux-gnu --release --no-default-features --features static_ssl

    rm -r "$AARCH64_NOGUI_PACKAGE_FOLDER" || true
    mkdir -p "$AARCH64_NOGUI_PACKAGE_FOLDER"
    cp ./target/aarch64-unknown-linux-gnu/release/noaa-apt "$AARCH64_NOGUI_PACKAGE_FOLDER/"
    cp -r "./test" "$AARCH64_NOGUI_PACKAGE_FOLDER/"
    cp -r "./res" "$AARCH64_NOGUI_PACKAGE_FOLDER/"
    rm -r "$AARCH64_NOGUI_PACKAGE_FOLDER/test/results" || true

    pushd "$AARCH64_NOGUI_PACKAGE_FOLDER/"
    zip -rq "$PACKAGES_FOLDER/$AARCH64_NOGUI_PACKAGE_NAME.zip" ./*
    popd

fi
