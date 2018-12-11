---
title: Usage
layout: main
---

## Usage

### GUI

Run by clicking the executable, or from terminal without arguments. You can do
two things:

- Decode a WAV file into a PNG.

- Resample a WAV into another WAV, this is useful if you want to try a program
  like [atp-dec/apt-dec] that requires a specific sample rate.

![GUI](./images/gui.png)

### Terminal

```
$ ./noaa-apt --help

Usage:
    ./target/release/noaa-apt [OPTIONS] [INPUT_FILENAME]

Decode NOAA APT images from WAV files. Run without arguments to launch the GUI

positional arguments:
  input_filename        Input WAV file.

optional arguments:
  -h,--help             show this help message and exit
  -d,--debug            Print debugging messages.
  -q,--quiet            Don't print info messages.
  -o,--output FILENAME  Set output path. When decoding images the default is
                        './output.png', when resampling the default is
                        './output.wav'.
  -r,--resample SAMPLE_RATE
                        Resample WAV file to a given sample rate, no APT image
                        will be decoded.
```
