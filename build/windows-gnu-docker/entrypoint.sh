#!/bin/sh

set -e
set -x

PACKAGE_FOLDER=/home/rustacean/src/target/x86_64-pc-windows-gnu/package/
# GTK_INSTALL_PATH=/usr/i686-w64-mingw32
GTK_INSTALL_PATH=/usr/x86_64-w64-mingw32/sys-root/mingw/

/home/rustacean/.cargo/bin/cargo build --target=x86_64-pc-windows-gnu --release

rm -r "$PACKAGE_FOLDER" || true
mkdir -p "$PACKAGE_FOLDER"

# Copy exe
cp ./target/x86_64-pc-windows-gnu/release/*.exe "$PACKAGE_FOLDER"

# Copy GTK files
cp "$GTK_INSTALL_PATH"/bin/*.dll "$PACKAGE_FOLDER"
mkdir -p "$PACKAGE_FOLDER/share/glib-2.0/schemas"
mkdir "$PACKAGE_FOLDER/share/icons"
cp "$GTK_INSTALL_PATH"/share/glib-2.0/schemas/* "$PACKAGE_FOLDER/share/glib-2.0/schemas"
cp -r "$GTK_INSTALL_PATH"/share/icons/* "$PACKAGE_FOLDER/share/icons"

# Copy settings.ini
mkdir "$PACKAGE_FOLDER/share/gtk-3.0"
cp ./build/windows-files/settings.ini "$PACKAGE_FOLDER/share/gtk-3.0/"

# Copy test files
mkdir "$PACKAGE_FOLDER/test"
cp ./test/*.wav "$PACKAGE_FOLDER/test/"
