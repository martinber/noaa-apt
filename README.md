# noaa-apt

NOAA APT image decoder.

Doesn't do anything special, takes a recorded WAV file (from GQRX, SDR#, etc.)
and decodes the raw image. Later you can rotate the image and adjust the
contrast with something like GIMP or Photoshop.

Works with WAV files of any sample rate, 32 bit float or 16 bit integer encoded.

Written in Rust as a learning exercise but could be useful to someone. Never
used Rust or made a GUI before. If you get some kind of error or bad result
don't hesitate to open a Issue here or to send me an email. You can try to run
the program with the `--debug` option for more info.

## Usage

### GUI

Run by clicking the executable, or from terminal without arguments.

You can decode to a PNG file or resample audio to WAV.

![GUI](./extra/gui.png)

### On terminal

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

## Compile

**Build with `--release`, Rust does some optimizations and it works MUCH
faster. Really, otherwise it takes FOREVER.**

```
cargo build --release
```

## Test

```
cargo test
```

If you get something like a wall of errors because linking with GSL fails, run
with the ``GSLv2`` feature:

```
cargo test --features GSLv2
```

## Example

From a WAV file I found somewhere on Internet, the US upside down:

![Example image](./extra/example.png)

The output is upside down if the satellite went from south to north instead of
north to south that day.

## Alternatives

Just bought an RTL-SDR and tried to receive NOAA APT images, I'm new to this but
as of August 2018:

- [WXtoImg], by far the most popular, lots of features but the site looks dead
  forever.

- [WXtoImg Restored], unofficial mirror with installers recovered by users.

- [atp-dec/apt-dec], works really good. Keep in mind that the [1.7 release]
  looks newer than the [repo's master branch]. I tried several times to compile
  the [repo's master branch] without success, later I realized that there was a
  newer [1.7 release] and it worked.

- [zacstewart/apt-decoder], written in Python, slower than the others but really
  simple. Doesn't align the image to the sync stripes.

- [martinber/apt-decoder], bad hack made by me on top of
  [zacstewart/apt-decoder] trying to align the image to the sync stripes. Still
  slow and minor artifacts on the image if you look at the vertical stripes.

[WXtoImg]: http://wxtoimg.com/
[WXtoImg Restored]: https://wxtoimgrestored.xyz/
[atp-dec/apt-dec]: https://github.com/csete/aptdec
[1.7 release]: https://github.com/csete/aptdec/releases
[repo's master branch]: https://github.com/csete/aptdec
[zacstewart/apt-decoder]: https://github.com/zacstewart/apt-decoder
[martinber/apt-decoder]: https://github.com/martinber/apt-decoder

## Dependencies

- Development:

  - GNU Scientific Library, only for running the tests:

    - Linux: `sudo apt install libgsl0-dev libgsl0`.

    - In Windows: Never did test there.

  - GTK:

    - Linux: `sudo apt install libgtk-3-dev`.

    - In Windows: TODO.

## Things I should do

- Add binaries.

- Support Windows.

- The parameters used for filter design are hardcoded.

- Do tests.

- Separate GUI and no GUI builds.

## Algorithm

AM resampling and demodulation using FIR filter, following method 4 or 5 in
reference [1]:

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

- Demodulate AM signal yo get the APT signal.

  - Get hilbert filter impulse response by window method.

    - Get kaiser window.

    - Sample and window the function `1/(pi*n) * (1-cos(pi*n))`.

  - Get the imaginary part of the Analytical Signal by doing the convolution
    with the hilbert impulse response. This part adds a delay (maybe I should
    fix that).

  - Get the real part of the Analytical Signal by adding the same delay to the
    original AM signal.

  - Calculate the absolute value of each sample: `sqrt(real^2 + imag^2)`.

- Find the position of the sync frames of the APT signal (the white and black
  stripes that you can see in the final image).

  - Calculate the cross correlation between a hardcoded sync frame and the APT
    signal.

  - The peaks of that cross correlation show the locations of the sync frames in
    the APT signal.

- Map the values of the signal to numbers between 0 and 255.

- Generate the final image starting a new line on every sync frame.

## Resampling algorithm

I did something like what you can see
[here](https://ccrma.stanford.edu/~jos/resample/) but with a easier
implementation.

![Resampling algorithm](./extra/resampling.png)

For each output sample, we calculate the sum of the products between input
samples and filter coefficients.

## Notes

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
  AM demodulation methods.

- [Hilbert Transform Design Example][3]: How to get the analytic signal.

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

[analytic signal]: https://en.wikipedia.org/wiki/Analytic_signal
