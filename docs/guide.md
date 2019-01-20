---
title: Guide
layout: main
---

---

**Contents**

- TOC
{:toc}

---

**TODO**: Guide about antennas, GQRX, etc.

<!--
Keep in mind that the output is upside down if the satellite went from south to
north instead of north to south that day.
-->

## Notes

- These satellites send images at realtime, line by line. Something like a
  1000x1 resolution video or like a flying scanner. These satellites fly on a
  polar orbit (north to south and south to north).

- You don't need a LNA or a high antenna. I'm using a Double Cross Antenna,
  roughly 2m above the ground and 3-4m of RG-58 coax. Probably helps being far
  from big cities.

- For satellite tracking I use gpredict. Otherwise I use the Heavens-Above Android App/Website.

- For NOAA, first I record a WAV file on GQRX using the following settings. I
  don't know if these are the best settings, but it should work.

    - Demodulation: WFM (mono).

    - Filter Width: Custom, just wide enough so the signal fits as you can see
      on the spectrum analyzer.

    - Filter Shape: Normal.

    - AGC: off.

    - Noise Blanker: No.

    - Squelch: Disabled (-150dB).

    - Maximum LNA gain slider.

    - I don't use any of "DC remove", "I/Q Balance", etc.

    - Input rate: 1800000

    - Decimation: None.

### Frequencies

- NOAA 15: 137.62MHz.

- NOAA 18: 137.9125MHz.

- NOAA 19: 137.1MHz.
