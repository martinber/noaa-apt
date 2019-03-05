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

## Really important things to know

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

## About how this thing works

- These satellites send images at realtime, line by line. Something like a
    1000x1 resolution video or like a flying
    [image scanner](https://en.wikipedia.org/wiki/Image_scanner). These
    satellites fly on a polar orbit (north to south and south to north).

- The satellite sends a FM signal, something like the FM radio transmissions.
    Instead of music it sends images, but it works exactly the same, actually
    you hear the thansmissions when you demodulate the FM signal. You can
    imagine that if your car stereo could tune to 137.1MHz you should hear the
    satellite tranmsissions.
    That's also why you can save the recordings as .wav files and decode those
    .wav files with this program, because it's just a sound that encodes an
    image.

- If you want to know more visit [How it works](./how-it-works.html).

## Notes

**Work in progress**.

- Audio sample on [SigidWiki](https://www.sigidwiki.com/wiki/Automatic_Picture_Transmission_(APT))

- You don't need a LNA or a high antenna. I'm using a Double Cross Antenna,
  roughly 2m above the ground and 3-4m of RG-58 coax. Probably helps being far
  from big cities.

- For satellite tracking I use gpredict. Otherwise I use the Heavens-Above Android App/Website.

- You don't have to compensate doppler shift.

- For NOAA, first I record a WAV file on GQRX using the following settings. I
  don't know if these are the best settings, but it should work.

    - Demodulation: WFM (mono).

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

- IF noise reduction in SDR#

https://old.reddit.com/r/RTLSDR/comments/atw4cf/how_i_received_my_first_noise_free_noaa_images/

### Frequencies

- NOAA 15: 137.62MHz.

- NOAA 18: 137.9125MHz.

- NOAA 19: 137.1MHz.

### WXtoIMG guide

- Resample the recording to 11025Hz.
- Enable _Expert mode_, on _Options > GUI Options_.
- Restart WXtoIMG.
- _File > Load_ audio file.
- Select _Satellite > NOAA_.
- Select _Options > Disable map overlay_ because it's going to be wrong unless you correct the timestamp of the file.
- _File > Decode_.

- _Options > Resync_.
- _Options > Disable PLL_.
- _Options > Illumination Compensation_.

https://www.wraase.de/download/wxtoimg/wxgui.pdf
