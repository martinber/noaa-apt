---
title: Alternatives
layout: main
---

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
