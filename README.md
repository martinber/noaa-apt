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
