---
title: Guide
layout: main
---

---

**Contents**

- TOC
{:toc}

---

## Introduction

This guide is meant for people that just heard about NOAA satellites, that just
saw somewhere that you can receive images from satellites quite easily and
cheap. Read this get started.

This is a guide about how to receive images, if you want information about how
to use noaa-apt check the [Usage](./usage.html) page.

## Important things to know

- NOAA **images are black and white**. People on the internet share color
    images, those images were originally black and white and then were
    colorized, probably by WXtoImg.

- NOAA **images don't have map lines** (divisions between countries, states,
    or coastlines). When you see on the internet images with lines, it means
    that they used WXtoImg to draw them according to a map and calculations
    about the position of the satellite when the image was taken.

- The **images are upside down 50% of the time**, that's because the satellites
    sometimes go from south to north and sometimes fron north to south.
    WXtoImg calculates the orbit of the satellites and rotates the image
    accordingly.

- Images look **much better on daylight**, the satellites also send infrared
    images but I recommend midday passes.

- These satellites send images at realtime, line by line. Something like a
    1000x1 resolution video or like a flying
    [image scanner](https://en.wikipedia.org/wiki/Image_scanner). These
    satellites fly on a polar orbit (north to south and south to north).

- The satellite sends a FM signal, something like FM broadcast radio
    transmissions. Instead of music it sends images, but it works exactly the
    same. You can imagine that if your car stereo could tune to 137.1MHz you
    should hear the satellite transmissions. That's also why you can save the
    recordings as .wav files and decode those .wav files with this program,
    because it's just a sound that encodes an image.

- If you want to know more visit [How it works](./how-it-works.html).

## Guide

**Work in progress**.

Maybe this image helps to understand how everything works, maybe not:

![Diagram]({{ site.baseurl }}/images/diagram.png)

You need a RTL-SDR, it looks like a USB drive but it has a connector for
antennas too. You can try first with a
[V-dipole antenna](https://www.rtl-sdr.com/simple-noaameteor-weather-satellite-antenna-137-mhz-v-dipole/),
RG-58 coax cable and adapters. Later you can improve reception if necessary
building a
[Double Cross antenna](https://www.rtl-sdr.com/instructions-for-building-a-double-cross-antenna-great-for-noaameteor-weather-satellites/)
or a QFH antenna.

First you have to download some SDR software, for example GQRX (GNU/Linux) or
SDR# (Windows). With it you can tune your SDR to any frequency and demodulate FM
signals, try it with broadcast radio first.

To track the satellites positions you can use for example gpredict (GNU/Linux)
or Heavens-Above (online or Android). You should look for passes during the day
with at least 10° of max elevation.

Set your SDR software for FM demodulation and tune it to the correct frequency
depending on the satellite:

- NOAA 15: 137.62MHz.

- NOAA 18: 137.9125MHz.

- NOAA 19: 137.1MHz.

I use the following configuration on GQRX. I don't know if these are the best
settings, but it should work.

- Demodulation: WFM (Wide FM, mono).

- Filter Width: Custom, just wide enough so the signal fits as you can see
  on the spectrum analyzer.

- Filter Shape: Normal.

- AGC: off.

- Noise Blanker: No.

- Squelch: Disabled (-150dB).

- LNA gain: Max, but you can play with it and guess where it has the best
    signal to noise ratio.

- I don't use any of "DC remove", "I/Q Balance", etc.

- Input rate: 1800000

- Decimation: None.

When the satellite is passing start recording a WAV file, you should hear the
sound of the demodulated FM signal. When finished open the WAV file on noaa-apt
to decode the image.

Here you can see a screenshot of GQRX, things to note:

- I have "Freq zoom" at 4x so I can see the signal better on the spectrum
    analyzer, these signals have a very narrow bandwidth.

- The signal should fit inside the filter bandwidth (grey area).

- I'm recording the WAV file (the "Rec" button on the bottom right is pressed).

![GQRX]({{ site.baseurl }}/images/gqrx.png)

## Notes

- [Example of a WAV file]({{ site.baseurl }}/examples/argentina.wav), hear it,
    near the middle of the recording you can hear the ticking sound of APT
    signals.

- You don't need a LNA or a high antenna. I'm using a Double Cross Antenna,
    roughly 2m above the ground and 3-4m of RG-58 coax. Probably helps being far
    from big cities.

- Its not necessary to compensate doppler shift.

## WXtoIMG guide

If you want to decode your WAV file on WXtoIMG:

- Resample the recording to 11025Hz,
    [you can use noaa-apt for that](./usage.html).

- Enable _Expert mode_, on _Options > GUI Options_.

- Restart WXtoIMG.

- _File > Load_ audio file.

- Select _Satellite > NOAA_.

- Select _Options > Disable map overlay_ because it could be wrong if you have a
    wrong timestamp on your file.

- _File > Decode_.

If you want map overlay:

- The modification timestamp of the new file should be the moment of the end of
    the pass. If you resampled the file using noaa-apt it should be OK,
    otherwise change it if necessary using the noaa-apt GUI or using your
    terminal.

- Uncheck _Options > Disable map overlay_.

- Set _Options > Ground Station Location_.

- Use _Enhacements > Normal_ or any enhacement different than _Pristine_
    (because it means no enhacements).

Optional:

- _Options > Resync_.

- _Options > Disable PLL_.

- _Options > Illumination Compensation_.

## References

- [WXtoIMG guide][1]

[1]: https://www.wraase.de/download/wxtoimg/wxgui.pdf
