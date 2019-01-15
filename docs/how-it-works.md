---
title: How it works
layout: main
---

- TOC
{:toc}

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

### About frequency

![Frequency unit comparison]({{ site.baseurl }}/images/frequency.png)

I made this drawing because I keep forgetting how to represent frequencies. Here
you can see the frequency spectrum of some APT signal sampled at 11025Hz, the
peak is the AM carrier at 2400Hz, and everything wraps around because we use
discrete signals.

### Resampling algorithm

I did something like what you can see
[here](https://ccrma.stanford.edu/~jos/resample/) but with a easier
(and slower) implementation.

![Resampling algorithm]({{ site.baseurl }}/images/resampling.png)

For each output sample, we calculate the sum of the products between input
samples and filter coefficients.

### AM demodulation

Previously I used a Hilbert filter to get the [analytic signal], because the
absolute value of the [analytic signal] is the modulated signal.

Then I found a very fast demodulator implemented on [pietern/apt137]. For each
output sample, you only need the current input sample, the previous one and the
carrier frequency:

![AM demodulation formula]({{ site.baseurl }}/images/demodulation.png)

I couldn't find the theory behind that method, looks similar to I/Q
demodulation. I was able to reach that final expression (which is used by
[pietern/apt137]) by hand and I wrote the steps on ``extra/demodulation.pdf``. I
think it only works if the input AM signal is oversampled, maybe that's why I
can't find anything about it on the web.

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

- [APT on sigidwiki.com][16]: More about the APT format.

- [github-markdown-toc generator][17]: I'm using that for the table of contents
  generation.

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
[16]: https://www.sigidwiki.com/wiki/Automatic_Picture_Transmission_(APT)
[17]: https://github.com/ekalinin/github-markdown-toc

[WXtoImg]: http://wxtoimg.com/
[analytic signal]: https://en.wikipedia.org/wiki/Analytic_signal
[pietern/apt137]: https://github.com/pietern/apt137
