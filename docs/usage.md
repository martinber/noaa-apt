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

When using a Raspberry Pi, I recommend the "fast" profile, you can enable it
using `-p fast` or editing the
[configuration file](./usage.html#configuration-file).

### GUI

Run by clicking the executable, or from terminal without arguments.

On _Tools > Resample WAV_ you can resample a WAV into another WAV, this is
useful if you want to try a program like [WXtoIMG] or [atp-dec/apt-dec] that
requires a specific sample rate. If resampling, the modification timestamp
should be preserved correctly.

On _Tools > Timestamp WAV_ you can change the modification date and time present
on the metadata of a file. Useful when you want to decode a WAV file on
[WXtoIMG] and you need to change the timestamp to fix the map overlay. You can
load a timestamp from another file in the case you want to copy the timestamp
from one file to another. Otherwise just select time, date and write the
timestamp to your WAV recording.

![GUI]({{ site.baseurl }}/images/gui.png)

If you are having problems, you can try running noaa-apt from a console. On
GNU/Linux run `noaa-apt` from your terminal, on Windows double click
`noaa-apt-console.exe` or run it from Powershell.

### Terminal

On GNU/Linux run `noaa-apt` from your terminal. On Windows you should use
`noaa-apt-console.exe` to be able to see console output.

If you run the program without arguments the GUI will open, so make sure to at
least give the input filename as an option.

```
Usage:
  noaa-apt [OPTIONS] [INPUT_FILENAME]

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
  -p,--profile PROFILE  Profile to use, values loaded from settings file.
                        Possible values: "standard", "fast" or "slow".
  -o,--output FILENAME  Set output path. When decoding images the default is
                        './output.png', when resampling the default is
                        './output.wav'.
  -r,--resample SAMPLE_RATE
                        Resample WAV file to a given sample rate, no APT image
                        will be decoded.
```

If resampling, the modification timestamp should be preserved correctly.

The timestamp modification tool is only available via the GUI, if you need to
set arbitrary timestamps to files using the terminal use your OS commands. On
GNU/Linux:

- Generally you just have to copy the timestamp from one file to another
    one using: `touch -r reference_file.wav recording.wav`.

- Set any timestamp you want with
    `touch -d "2019-01-31 18:31:20.579283000" recording.wav`.

## Advanced settings

### Disable syncing

The program aligns the image to sync frames (the black and white stripes),
disabling can help with some noisy images, but the resulting image has some
slant. See also [Troubleshooting](./usage.html#troubleshooting).

![Comparison between synced and not synced image]({{ site.baseurl }}/images/syncing.jpg)

### Contrast adjustment

You can choose between three contrast adjustment methods:

- MinMax: It doesn't do anything, just maps the darkest pixel to black and the
    brightest pixel to white.

- 98 percent: I don't know how to name it, ignores the darkest 1% of pixels and
    the brightest 1% of pixels. Something like a percentile, this is the
    default method.

- From telemetry: Checks the wedges from telemetry bands, those wedges have
    shades of grey that go from black to white. This method is better than "98
    percent" but can fail on noisy images.

### Export WAV steps

If enabled, the program will save lots of WAV files, one for each step done on
the decoding process. I open those files on Audacity for debugging, check if the
filters are working, etc. The directory where the files are saved is the
working directory of the program, generally your home folder.

Exporting the "resample filtered" is a very expensive operation, can take
several GiB of both RAM and disk, so this step is not exported by default and
has to be enabled separately.

### Profile

Only available as a commandline option, but you can change the default profile
by editing the [configuration file](./usage.html#configuration-file). On
Raspberry Pi I recommend using the "fast" profile. If you are having noisy
images you can try the "slow" profile once just in case, but the "standard"
profile should always work fine.

### Configuration file

The first time you open noaa-apt, a default configuration file will be created
on `~/.config/noaa-apt/settings.toml` or
`C:\Users\[USER]\AppData\Roaming\noaa-apt\settings.toml` depending on your
operating system.

There you can disable the update check, select the default profile to use (fast,
standard or slow), or edit those profiles.

## Troubleshooting

### Problems with noaa-apt

If the program crashes or you want more information, run noaa-apt with console
output, see above.

### Upside down images

These satellites have polar orbits, so sometimes you see them go from north to
south and sometimes from south to north. If the satellite went from south to
north you should rotate the image.

### Bad contrast, dark images

The program tries to adjust the contrast with a conservative method, you can try
another one, check the advanced settings.

If the image needs more contrast you can adjust it with an image editor.
I use GIMP and the tool _Colors > Levels_. You can pick a white spot and a black
spot as you can see on this screenshot or you can move the _Input Levels_
sliders manually.

![Contrast correction using GIMP]({{ site.baseurl }}/images/contrast.jpg)

Also, images taken during the night (taken using a infrared sensor) or close to
sunrise/sunset have very little contrast, I recommend to take images during the
day with the sun high above the horizon.

Check these images for a comparison. On the left image ignore the noise (caused
by a bad antenna) and note how the right channel changed the active sensor from
visible to infrared and went from a dark to a bright image.

![Comparison between night and day images]({{ site.baseurl }}/images/night_day.jpg)

### Syncing problems

This program starts a new line when it receives a sync frame (those seven white
and black stripes), works well if the signal has clear sync frames but can
produce horizontal lines on some images.

You can disable the syncing (on GUI there is a checkbox, for commandline the
option is `--no-sync`). The image should have a
[smooth slant](./usage.html#disable-syncing) and you can manually edit and
straighten the image. If without syncing the image looks worse, you have missing
samples, see below.

![Example of syncing problems]({{ site.baseurl }}/images/disable_sync.jpg)

### Missing samples

Sometimes the computer has hiccups and skips samples when receiving and creating
the WAV file (this is not caused by noaa-apt), this can cause short horizontal
black lines or syncing problems (producing long horizontal lines).

You can try:

- Receiving on another computer.
- Using a smaller RF sample rate 1,000,000 (1Msps) should be fine.
- Closing unused programs.
- Giving more priority to the SDR receiver process.
- Increasing the audio buffer, I am aware that at least SDR-Console has this
    setting. On SDR# I think that increasing the _Latency_ setting on the
    _Audio_ panel should give the same result.

If you think that your receiver might be skipping samples, you can try disabling
syncing, you should see a mess instead of a
[smooth slant](./usage.html#disable-syncing), thank you _xxretartistxx_ and
_unknownantipatriot_ for these images:

![Example of missing samples]({{ site.baseurl }}/images/missing_samples.jpg)

![Another example of missing samples]({{ site.baseurl }}/images/missing_samples_2.jpg)

### Noise

Sometimes the reception is bad and you should try receiving another pass, maybe
with a better antenna.

![Example of noise]({{ site.baseurl }}/images/noise.jpg)

### Something strange

You can set the option to export a WAV file with the samples used in each step
of the decoding process, then open each WAV on something like Audacity to see
where things went wrong. I use Audacity because it shows the samples clearly and
_Analyze > Plot Spectrum_ is really useful.

Otherwise you can send me the WAV file so I can take a look, it could be a
problem with my decoder.

[atp-dec/apt-dec]: https://github.com/csete/aptdec
[WXtoImg]: http://wxtoimg.com/
