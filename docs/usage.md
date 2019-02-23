---
title: Usage
layout: main
---

---

**Contents**

- TOC
{:toc}

---

## Program usage

Takes a recorded WAV file (from GQRX, SDR#, etc.) and decodes the raw image.
Later you can rotate the image and adjust the contrast with something like GIMP
or Photoshop.

Works with WAV files of any sample rate, 32 bit float or 16 bit integer encoded.
When loading audio files with more than one channel (stereo), only the first one
is used.

### GUI

Run by clicking the executable, or from terminal without arguments. You can do
two things:

- Decode a WAV file into a PNG.

- Resample a WAV into another WAV, this is useful if you want to try a program
  like [atp-dec/apt-dec] that requires a specific sample rate.

![GUI]({{ site.baseurl }}/images/gui.png)

### Terminal

```
$ ./noaa-apt --help
Usage:
  ./target/debug/noaa-apt [OPTIONS] [INPUT_FILENAME]

Decode NOAA APT images from WAV files. Run without arguments to launch the GUI

Positional arguments:
  input_filename        Input WAV file.

Optional arguments:
  -h,--help             Show this help message and exit
  -v,--version          Show version and quit.
  -d,--debug            Print debugging messages.
  -q,--quiet            Don't print info messages.
  --wav-steps           Export a WAV for every step of the decoding process for
                        debugging, the files will be located on the current
                        folder, named {number}_{description}.wav
  --export-resample-filtered
                        Export a WAV for the expanded and filtered signal on
                        the resampling step. Very expensive operation, can take
                        several GiB of both RAM and disk. --wav-steps should be
                        set.
  --no-sync             Disable syncing, useful when the sync frames are noisy
                        and the syncing attempts do more harm than good.
  -c,--contrast CONTRAST
                        Contrast adjustment method for decode. Possible values:
                        "98_percent", "telemetry" or "disable". 98 Percent used
                        by default.
  -o,--output FILENAME  Set output path. When decoding images the default is
                        './output.png', when resampling the default is
                        './output.wav'.
  -r,--resample SAMPLE_RATE
                        Resample WAV file to a given sample rate, no APT image
                        will be decoded.
```

## Advanced settings

TODO

## Troubleshooting

### Upside down images

These satellites have polar orbits, so sometimes you see them go from north to
south and sometimes from south to north. If the satellite went from south to
north you should rotate the image.

### Bad contrast, dark images

The program tries to adjust the contrast with a conservative method, you can try
another one, check the advanced settings.

If the image needs more contrast you can adjust it with an image editor.
I use GIMP and the tool _Colors > Levels_. You can pick a white spot and a black
spot as you can see on this screenshot.

![Contrast correction using GIMP]({{ site.baseurl }}/images/contrast.jpg)

### Syncing

This program starts a new line when it receives a sync frame (those seven white
and black stripes), works well if the signal has clear sync frames.

The first time I recorded a NOAA APT signal using a FM amateur radio the bright
parts had lot's of noise (had saturation when receiving white, I don't know
why), the sync frames were really low quality and the alignment was really bad.
Anyways, when using SDR that doesn't happen.

Every decoder I've tested, excluding [WXtoIMG], has the same problem.

You can disable the syncing (on GUI there is a checkbox, for commandline the
option is `--no-sync`). Then you should manually edit and straighten the image.

![Example of syncing problems]({{ site.baseurl }}/images/disable_sync.jpg)

### Noise

Sometimes the reception is bad and you should try receiving another pass, maybe
with a better antenna.

![Example of noise]({{ site.baseurl }}/images/noise.jpg)

### Something strange

You can set the option to export a WAV file with the samples used in each step
of the decoding process, then open each WAV on something like Audacity to see
where things went wrong. I use Audacity because it shows the samples clearly and
_Analyze > Plot Spectrum_ is really useful.

[atp-dec/apt-dec]: https://github.com/csete/aptdec
[WXtoImg]: http://wxtoimg.com/
