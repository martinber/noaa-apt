---
title: Guide
layout: main
---

---

**Contents**

- TOC
{:toc}

---

**Work in progress**.

## Notes

- Keep in mind that the output is upside down if the satellite went from south to
	north instead of north to south that day.

- Audio sample on [SigidWiki](https://www.sigidwiki.com/wiki/Automatic_Picture_Transmission_(APT))

- These satellites send images at realtime, line by line. Something like a
  1000x1 resolution video or like a flying scanner. These satellites fly on a
  polar orbit (north to south and south to north).

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
