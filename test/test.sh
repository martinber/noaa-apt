#!/usr/bin/env bash

# Does some integration tests and saves the results to ./results/, deletes
# everything there.
# If ../noaa-apt exists, it uses that executable. Otherwise uses
# ../target/release/noaa-apt.

set -eu -o pipefail

# Get script directory, https://stackoverflow.com/a/246128
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ]; do
  TEST_DIR="$( cd -P "$( dirname "$SOURCE" )" >/dev/null 2>&1 && pwd )"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE"
done
TEST_DIR="$( cd -P "$( dirname "$SOURCE" )" >/dev/null 2>&1 && pwd )"

# Parent and results dir
MAIN_DIR="$TEST_DIR/../"
RESULTS_DIR="$TEST_DIR/results/"

PROGRAM=0
# Get executable
if [ -x "$MAIN_DIR/noaa-apt" ]; then
    PROGRAM="$MAIN_DIR/noaa-apt"
    printf "Using /noaa-apt\n"
else
    PROGRAM="$MAIN_DIR/target/release/noaa-apt"
    printf "Using /target/release/noaa-apt\n"
fi

# Clean previous results and create results folder

rm -r "$RESULTS_DIR" || true
mkdir "$RESULTS_DIR"

# Run tests

"$PROGRAM" -v

# Show executed commands
set -x

"$PROGRAM" -q "$TEST_DIR/test_11025hz.wav" -o "$RESULTS_DIR/decoded_apt.png"
"$PROGRAM" -q "$TEST_DIR/noise_48000hz.wav" -o "$RESULTS_DIR/decoded_noise.png"
"$PROGRAM" -q "$TEST_DIR/test_11025hz.wav" -r 48000 -o "$RESULTS_DIR/upsampled_apt.wav"
"$PROGRAM" -q "$TEST_DIR/test_11025hz.wav" -r 6000 -o "$RESULTS_DIR/downsampled_apt.wav"
"$PROGRAM" -q "$TEST_DIR/test_11025hz.wav" -r 3675 -o "$RESULTS_DIR/decimated_apt.wav"
"$PROGRAM" -q "$TEST_DIR/noise_48000hz.wav" -r 80000 -o "$RESULTS_DIR/upsampled_noise.wav"
"$PROGRAM" -q "$TEST_DIR/noise_48000hz.wav" -r 11025 -o "$RESULTS_DIR/downsampled_noise.wav"
"$PROGRAM" -q "$TEST_DIR/test_11025hz.wav" --tle "$TEST_DIR/test_tle.txt" -s "noaa_19" \
		-R "auto" -m "yes" -t "2018-12-22T20:39:41-00:00" -o "$RESULTS_DIR/decoded_apt_map.png"
