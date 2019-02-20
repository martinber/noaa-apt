#!/usr/bin/env bash

# Generate windows icon. Give svg input and ico output as arguments
# Needs inkscape

# ICO creation from https://graphicdesign.stackexchange.com/a/110023
# Temporary dir creation from https://stackoverflow.com/a/34676160

set -eu -o pipefail

if [ "$#" -ne 2 ]; then
    echo "Give svg input and ico output filenames"
    exit 1
fi

svg="$1"
output="$2"

# Temp dir creation ############################################################

work_dir=$(mktemp -d)

# Check if tmp dir was created
if [[ ! "$work_dir" || ! -d "$work_dir" ]]; then
  echo "Could not create temp dir"
  exit 1
fi

# Deletes the temp directory
function cleanup {
  rm -rf "$work_dir"
  echo "Deleted temp working directory $work_dir"
}

# register the cleanup function to be called on the EXIT signal
trap cleanup EXIT

# Icon creation ################################################################

echo Exporting svg...

sizes=(16 32 48 256)

for i in ${sizes[@]}; do
  inkscape $svg --export-png="$work_dir/favicon-$i.png" -w$i -h$i --without-gui
done

# echo Compressing...

## Replace with your favorite (e.g. pngquant)
## Not really neccesary, size doesn't change much and result is too lossy
# optipng -o7 favicon-*.png
# pngquant -f --ext .png "$work_dir"/*.png --posterize 4 --speed 1

echo Converting to .ico...

convert $(ls -v "$work_dir"/favicon-*.png) "$output"

echo Done
