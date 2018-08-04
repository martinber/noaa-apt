# noaa-apt

Work in progress decoder for NOAA APT images from a recorded WAV file.

Written in Rust, never tried to do signal processing or to use Rust before...

## Alternatives

Just bought an RTL-SDR and tried to receive NOAA APT images, I'm new to this but
as of July 2018:

- [wxtoimg], by far the most popular, lots of features but the site looks dead
  forever, you can still get some binaries uploaded by some people if you are
  lucky.

- [atp-dec/apt-dec], works really good. Keep in mind that the [1.7 release]
  looks newer than the [repo's master branch]. I tried several times to compile
  the [repo's master branch] without success, later I realized that there was a
  newer [1.7 release] and it worked.

- [zacstewart/apt-decoder], written in Python, slower than the others but really
  simple. Doesn't align the image to the sync stripes.

- [martinber/apt-decoder], bad hack made by me on top of
  [zacstewart/apt-decoder] trying to align the image to the sync stripes. Still
  slow and minor artifacts on the image if you look at the vertical stripes.

[wxtoimg]: http://wxtoimg.com/
[atp-dec/apt-dec]: https://github.com/csete/aptdec
[1.7 release]: https://github.com/csete/aptdec/releases
[repo's master branch]: https://github.com/csete/aptdec
[zacstewart/apt-decoder]: https://github.com/zacstewart/apt-decoder
[martinber/apt-decoder]: https://github.com/martinber/apt-decoder

## Dependencies

- GNU Scientific Library:

  - `sudo apt install libgsl0-dev`.

- GNUPlot:

  - TODO


## Algorithms

AM resampling and demodulation using FFT (maybe too slow?):

- Load samples from WAV.
- Resample and get [analytic signal].
  - Get L (interpolation factor) and M (decimation factor).
  - Insert L-1 zeros between samples,
  - Calculate FFT.
    - Remove negative half of spectrum to get analytic signal.
    - Filter spectrum images on the right to interpolate.
    - Calculate IFFT.
  - Decimate removing M-1 samples.
- Get absolute value of analytic signal to finish AM demodulation.

AM resampling and demodulation using FIR filter, following method 4 or 5 in
reference [1]:

- Load samples from WAV.
- Resample and get [analytical signal].
  - Get L (interpolation factor) and M (decimation factor).
  - Insert L-1 zeros between samples,
  - Get filter from a common sample rate or generate a new one:
    - Calculate impulse response of hilbert filter and lowpass filter.
    - Calculate kaiser window from parameters or use a predefined one.
    - Multiply window with impoulse response for both filters.
  - Filter with lowpass to finish interpolation.
  - Decimate removing M-1 samples.
  - Get [analytic sygnal]:
    - Filter the signal.
    - Add the original signal.
- Get absolute value of analytic signal to finish AM demodulation.


## Analytical signal

For AM demodulation we use the [analytic signal].

### Hilbert filter

Frequency response: j for w < 0 and -j for w > 0.

Impulse response: `fs/(pi*n) * (1-cos(pi*n))`

For n=0, should be 0.

## Lowpass filter

Impulse response: `sin(n*wc)/(n*pi)`.

## Notes

- Looks like there are several definitions for Kaiser window values, I get
  different results compared to Matlab.

- I use 32 bit float and integers because it's enough?.

## References

- [Digital Envelope Detection: The Good, the Bad, and the Ugly][1]: Lists some
  AM demodulation methods.

- [Hilbert Transform Design Example][2]: How to get the analytic signal.

- [Spectral Audio Signal Processing: Digital Audio Resampling][3].

- [Impulse Response of a Hilbert Transformer][4].

- [Spectral Audio Signal Processing: Kaiser Window][5].

- [How to Create a Configurable Filter Using a Kaiser Window][6],

[1]: https://www.dsprelated.com/showarticle/938.php
[2]: https://www.dsprelated.com/freebooks/sasp/Hilbert_Transform_Design_Example.html
[3]: https://ccrma.stanford.edu/~jos/resample/
[4]: https://flylib.com/books/en/2.729.1/impulse_response_of_a_hilbert_transformer.html
[5]: https://ccrma.stanford.edu/~jos/sasp/Kaiser_Window.html
[6]: https://tomroelandts.com/articles/how-to-create-a-configurable-filter-using-a-kaiser-window

[analytic signal]: https://en.wikipedia.org/wiki/Analytic_signal
