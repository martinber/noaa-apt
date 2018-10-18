# noaa-apt

NOAA APT image decoder.

Takes a recorded WAV file (from GQRX, SDR#, etc.) and decodes the raw image.
Later you can rotate the image and adjust the contrast with something like GIMP
or Photoshop.

Works with WAV files of any sample rate, 32 bit float or 16 bit integer encoded.
When loading audio files with more than one channel, only the first one is used.

Written in Rust as a learning exercise but could be useful to someone. Never
used Rust or made a GUI before. If you get some kind of error or bad result
don't hesitate to open a Issue here or to send me an email. You can try to run
the program with the `--debug` option for more info.

## Usage

### GUI

Run by clicking the executable, or from terminal without arguments. You can do
two things:

- Decode a WAV file into a PNG.

- Resample a WAV into another WAV, this is useful if you want to try a program
  like [atp-dec/apt-dec] that requires a specific sample rate.

![GUI](./extra/gui.png)

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

## Download

You can download executables for Linux or Windows from the
[releases page](https://github.com/martinber/noaa-apt/releases). Your options
are:

- Linux:

  - Last version binary: Has GUI. Needs GTK and GLIBC version at least 2.19. I
    think that should work in most common distros.

  - Build yourself the last version.

  - Version 0.9.1 binary: Doesn't have GUI, only terminal. Should work
    everywhere.

- Windows:

  - Download binary for the last version.

  - Build yourself the last version (never tried to do that from Windows).

## Example

From a WAV file I found somewhere on Internet, the US upside down:

![Example image](./extra/example.png)

The output is upside down if the satellite went from south to north instead of
north to south that day.

## Compiling

### Linux

**Build with `--release`, Rust does some optimizations and it works MUCH
faster. Really, otherwise it takes FOREVER.**

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0).

- `sudo apt install libgtk-3-dev`.

- `cargo build --release`.

### Linux portable

I can't make `gtk-rs` to work with the `x86_64-unknown-linux-musl` target, so I'
building with the default `x86_64-unknown-linux-gnu` on Debian Jessie. I think
the binary works on any linux with GLIBC newer than the one used when building,
that's why I'm using a Debian Jessie docker image.

- Set up:

  - Install Docker.

  - `sudo apt install libgtk-3-dev`.

  - Move to root folder.

  - `docker build ./linux-docker/ -t noaa-apt-linux-build-image`.

  - `docker create -v $(pwd):/src --name noaa-apt-linux-build noaa-apt-linux-build-image`.

- Building the binary:

  - `docker start -ai noaa-apt-linux-build`.

  - The build is on `./target/x86_64-unknown-linux-gnu/`.

### Mac / OSX

**Build with `--release`, Rust does some optimizations and it works MUCH
faster. Really, otherwise it takes FOREVER.**

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0). The 'unix installer' is fine for Macs.

- Install dependencies via [Homebrew](https://brew.sh/):
  `brew install gtk+3 adwaita-icon-theme`.

- `cargo build --release`.

### Windows portable

I never tried to compile from Windows, I cross-compile from Linux to Windows. I
tried to get a mingw64-gtk environment to work on Debian without success. So I
use a Docker image I found
[here](https://github.com/LeoTindall/rust-mingw64-gtk-docker).

- Set up:

  - Install Docker.

  - `sudo apt install libgtk-3-dev`.

  - Move to root folder.

  - `docker build ./windows-docker/ -t noaa-apt-windows-build-image`.

  - `docker create -v $(pwd):/home/rustacean/src --name noaa-apt-windows-build noaa-apt-windows-build-image`.

- Building the package:

  - `docker start -ai noaa-apt-windows-build`.

  - The build is on `./target/x86_64-pc-windows-gnu/package/`.

## Alternatives

These are the alternatives I found, as of August 2018:

- [WXtoImg], by far the most popular, lots of features but the site looks dead
  forever.

- [WXtoImg Restored], unofficial mirror with installers recovered by users.

- [atp-dec/apt-dec], works really good. Keep in mind that the [1.7 release]
  looks newer than the [repo's master branch]. I tried several times to compile
  the [repo's master branch] without success, later I realized that there was a
  newer [1.7 release] and it worked.

- [pietern/apt137], written in C, extremely fast.

- [zacstewart/apt-decoder], written in Python, slower than the others but really
  simple. Doesn't align the image to the sync stripes.

- [martinber/apt-decoder], bad hack made by me on top of
  [zacstewart/apt-decoder] trying to align the image to the sync stripes. Still
  slow and minor artifacts on the image if you look at the vertical stripes.

- [ThatcherC/APT3000], written in JavaScript, looks very fast.

Others I found on GitHub:

- [brainwagon/noaa-apt], written in C, does not sync images.

- [LongHairedHacker/apt-decoder]. written in Rust.

- [dlew1716/APT], written in Python and C++, not easily usable.

- [toastedcornflakes/APT], written in Python, not easily usable.

- [la1k/wxfetch], fork of [atp-dec/apt-dec], I never tried it.

- [SopaXorzTaker/napt], written in C, can't figure out how to use it.

I measured the speed of most of them using the `time` utility from bash, and
made a comparison of the results on `./extra/comparison.ods`.

## Problems

### Syncing

This program starts a new line when it receives a sync frame (those seven white
and black stripes), works well if the signal has clear sync frames.

The first time I recorded a NOAA APT signal the bright parts had lot's of noise
(I think the FM demodulator bandwith was too narrow and had saturation when
receiving white), the sync frames were really low quality and the alignment was
really bad.

Every decoder I've tested, excluding [WXtoIMG], has the same problem.

## Tests

You need the GNU Scientific Library: `sudo apt install libgsl0-dev libgsl0`.

```
cargo test
```

If you get something like a wall of errors because linking with GSL fails, run
with the ``GSLv2`` feature:

```
cargo test --features GSLv2
```

## Things I should do

- Improve syncing performance.

- Option for disabling syncing.

- Separate thread for GUI.

- The parameters used for filter design are hardcoded.

- Do tests.

- Separate GUI and no GUI builds.

## How it works

### General

- Load samples from WAV.

- Resample to a intermediate sample rate: 20800Hz.

  - Get L (interpolation factor) and M (decimation factor) by looking at the
    greatest common divisor (GCD) between input and output sample rates.

  - Get interpolating lowpass filter inpulse response by window method.

    - Get kaiser window.

    - Sample and window the function `sin(n*cutout)/(n*pi)`.

  - Do the equivalent of:

    - Insertion of L-1 zeros between samples.

    - Filter by doing the convolution with the impulse response.

    - Decimate by M.

- Demodulate AM signal to get the APT signal.

  - Iterate over samples, get amplitude by looking at current and previous
    sample, see below.

- Find the position of the sync frames of the APT signal (the white and black
  stripes that you can see in the final image).

  - Calculate the cross correlation between a hardcoded sync frame and the APT
    signal.

  - The peaks of that cross correlation show the locations of the sync frames in
    the APT signal.

- Map the values of the signal to numbers between 0 and 255.

- Generate the final image starting a new line on every sync frame.

### Resampling algorithm

I did something like what you can see
[here](https://ccrma.stanford.edu/~jos/resample/) but with a easier
(and slower) implementation.

![Resampling algorithm](./extra/resampling.png)

For each output sample, we calculate the sum of the products between input
samples and filter coefficients.

### AM demodulation

Previously I used a Hilbert filter to get the [analytic signal], then the
absolute value of the [analytic signal] is the modulated signal.

Then I found a very fast demodulator implemented on [pietern/apt137]. For each
output sample, you only need the current input sample, the previous one and the
carrier frequency:

![AM demodulation formula](./extra/demodulation.png)

Where theta is the AM carrier frequency divided by the sample rate.

I couldn't find the theory behind that method, looks similar to I/Q
demodulation. I was able to reach that final expression (which is used by
[pietern/apt137]) by hand and I wrote the steps on ``extra/demodulation.pdf``. I
think it only works if the input AM signal is oversampled, maybe that's why I
can't find anything about it on the web.

### Notes

- Modulation:

  - The signal is modulated first on AM and then on FM.

  - FM frequencies:

    - NOAA 15: 137.62MHz.

    - NOAA 18: 137.9125MHz.

    - NOAA 19: 137.1MHz.

  - AM carrier: 2400Hz.

- APT signal:

  - 8 bits/pixel.

  - The signal amplitude represents the brightness of each pixel.

  - Two lines per second, 4160 pixels per second.

  - 2080 pixels per line, 909 useful pixels per line.

  - Each line has:

    - Sync A: Seven black and seven white pixels.

    - Space A: Some black pixels (periodically white ones too).

    - Image A: Visible/Infrared.

    - Telemetry A: For calibration I think?

    - Sync B: Some white and black pixels but I don't know the frequency.

    - Space B: Some white pixels (periodically black ones too).

    - Image B: Infrared.

    - Telemetry B: For calibration I think?

## References

- [NOAA Signal Decoding And Image Processing Using GNU-Radio][1]: About the APT
	image format.

- [Digital Envelope Detection: The Good, the Bad, and the Ugly][2]: Lists some
  AM demodulation methods. I'm not using any of these anyway.

- [Hilbert Transform Design Example][3]: How to get the analytic signal if using
  Hilbert filter demodulation.

- [Spectral Audio Signal Processing: Digital Audio Resampling][4].

- [Impulse Response of a Hilbert Transformer][5].

- [Spectral Audio Signal Processing: Kaiser Window][6].

- [How to Create a Configurable Filter Using a Kaiser Window][7],

- [Kaiser window approximation on StackOverflow][8]: I took the Bessel function
  from there (see the infinite sum), but I think that it's slightly wrong,
  according to [this][6] that minus sign should not be there. I'm comparing my
  implementation (without the minus sign) in my tests with `rgsl::bessel::I0`
  and everything works well, that's not the case when I add that minus sign. I
  suggested an edit on the StackOverflow post and the author said me that I'm
  wrong, so now I'm confused.

- [Error Handling in Rust][9].

- [Python GTK+ 3 Tutorial][10]: For Python but I like the Widget Gallery.

- [Cross-compiling from Ubuntu to Windows with Rustup][11].

- [How to compile C GTK3+ program in Ubuntu for windows?][12].

- [rust-mingw64-gtk Docker image][13]: I took the Windows Dockerfile from there.

- [zacstewart/apt-decoder][14]: Easy to understand NOAA APT decoder.

- [pietern/apt137][15]: The fastest NOAA APT decoder, I took the AM
  demodulation methid from there.


[WXtoImg]: http://wxtoimg.com/
[WXtoImg Restored]: https://wxtoimgrestored.xyz/
[atp-dec/apt-dec]: https://github.com/csete/aptdec
[1.7 release]: https://github.com/csete/aptdec/releases
[repo's master branch]: https://github.com/csete/aptdec
[zacstewart/apt-decoder]: https://github.com/zacstewart/apt-decoder
[martinber/apt-decoder]: https://github.com/martinber/apt-decoder
[ThatcherC/APT3000]: https://github.com/ThatcherC/APT3000
[brainwagon/noaa-apt]: https://github.com/brainwagon/noaa-apt
[LongHairedHacker/apt-decoder]: https://github.com/LongHairedHacker/apt-decoder
[dlew1716/APT]: https://github.com/dlew1716/APT
[toastedcornflakes/APT]: https://github.com/toastedcornflakes/APT
[la1k/wxfetch]: https://github.com/la1k/wxfetch
[pietern/apt137]: https://github.com/pietern/apt137
[SopaXorzTaker/napt]: https://github.com/SopaXorzTaker/napt


[1]: https://www.researchgate.net/publication/247957486_NOAA_Signal_Decoding_And_Image_Processing_Using_GNU-Radio
[2]: https://www.dsprelated.com/showarticle/938.php
[3]: https://www.dsprelated.com/freebooks/sasp/Hilbert_Transform_Design_Example.html
[4]: https://ccrma.stanford.edu/~jos/resample/
[5]: https://flylib.com/books/en/2.729.1/impulse_response_of_a_hilbert_transformer.html
[6]: https://ccrma.stanford.edu/~jos/sasp/Kaiser_Window.html
[7]: https://tomroelandts.com/articles/how-to-create-a-configurable-filter-using-a-kaiser-window
[8]: https://dsp.stackexchange.com/questions/37714/kaiser-window-approximation/37715#37715
[9]: https://blog.burntsushi.net/rust-error-handling/
[10]: https://python-gtk-3-tutorial.readthedocs.io/en/latest/index.html
[11]: https://www.reddit.com/r/rust/comments/5k8uab/crosscompiling_from_ubuntu_to_windows_with_rustup/
[12]: https://askubuntu.com/questions/942010/how-to-compile-c-gtk3-program-in-ubuntu-for-windows
[13]: https://github.com/LeoTindall/rust-mingw64-gtk-docker
[14]: https://github.com/zacstewart/apt-decoder
[15]: https://github.com/pietern/apt137

[analytic signal]: https://en.wikipedia.org/wiki/Analytic_signal
