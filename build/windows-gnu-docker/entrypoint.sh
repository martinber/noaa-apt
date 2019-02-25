#!/bin/sh

set -eux -o pipefail

# Get noaa-apt version from Cargo.toml
NOAA_APT_VERSION=$(awk '/^version =/{print substr($NF, 2, length($NF)-2)}' Cargo.toml)

PACKAGES_FOLDER=/home/rustacean/src/target/docker_builds
GUI_PACKAGE_NAME="noaa-apt-$NOAA_APT_VERSION-x86_64-windows-gnu"
GUI_PACKAGE_FOLDER="$PACKAGES_FOLDER/$GUI_PACKAGE_NAME"
# GTK_INSTALL_PATH=/usr/i686-w64-mingw32
GTK_INSTALL_PATH=/usr/x86_64-w64-mingw32/sys-root/mingw/

/home/rustacean/.cargo/bin/cargo build --target=x86_64-pc-windows-gnu --release

rm -r "$GUI_PACKAGE_FOLDER" || true
mkdir -p "$GUI_PACKAGE_FOLDER"

# Copy exe
cp ./target/x86_64-pc-windows-gnu/release/*.exe "$GUI_PACKAGE_FOLDER"

# Copy GTK files
cp "$GTK_INSTALL_PATH"/bin/*.dll "$GUI_PACKAGE_FOLDER"
mkdir -p "$GUI_PACKAGE_FOLDER/share/glib-2.0/schemas"
mkdir "$GUI_PACKAGE_FOLDER/share/icons"
cp "$GTK_INSTALL_PATH"/share/glib-2.0/schemas/* "$GUI_PACKAGE_FOLDER/share/glib-2.0/schemas"
cp -r "$GTK_INSTALL_PATH"/share/icons/* "$GUI_PACKAGE_FOLDER/share/icons"

# Copy settings.ini
mkdir -p "$GUI_PACKAGE_FOLDER/share/gtk-3.0"
cp ./build/windows-files/settings.ini "$GUI_PACKAGE_FOLDER/share/gtk-3.0/"

# Copy test files
mkdir "$GUI_PACKAGE_FOLDER/test"
cp ./test/*.wav "$GUI_PACKAGE_FOLDER/test/"

# Zip

pushd "$GUI_PACKAGE_FOLDER/"
zip -r "$PACKAGES_FOLDER/$GUI_PACKAGE_NAME.zip" ./*
popd
