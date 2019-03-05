---
title: Development
layout: main
---

---

**Contents**

- TOC
{:toc}

---

[The repository is available on GitHub](https://github.com/martinber/noaa-apt).

The available documentation is:

- Everything on this website, built from the `/docs` folder on the repository.

- Rustdoc documentation, generated from docstrings, built using `cargo doc
  --document-private-items` and available on `/target/doc/noaa-apt/index.html`

- Also there are a lot of comments on the code because I tend to forget
  everything quite fast.

## Code style

- Should follow the [Style guidelines] but 80 characters as line width.

- Docstrings: Try to document everything, follow [RFC-1574] without using links
  or examples.

- Order of `use` in code:

    ```
    extern crate thirdparty;
    pub mod ...;
    mod ...;
    pub use std::...;
    pub use thirdparty::...;
    pub use internal::...;
    use std::...;
    use thirdparty::...;
    use internal::...;
    ```

[Style guidelines]: https://doc.rust-lang.org/1.0.0/style/
[RFC-1574]: https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html#appendix-a-full-conventions-text

## Website

I'm using Jekyll, the website is built automatically by GitHub from the `/docs`
folder. These are the steps to build the website locally:

```
sudo apt-get install ruby-dev
gem install bundler jekyll
cd docs
jekyll build --baseurl "$(pwd)/_site/"
```

Notes:

- I'm using a modification of
    [Horizons-Jekyll-Theme](https://github.com/old-jekyll-templates/Horizons-Jekyll-Theme).

- Favicons generated using
    [RealFaviconGenerator](https://realfavicongenerator.net/)

- Apparently, the theme used _font-awesome_ to provide icons, I got rid of that.

- Changed font to Open Sans and now I'm loading from `default.html` instead of
  `style.css` because it's faster.

## Things to do

- Roadmap for 1.0.0:

    - Add warnings for short images when reading telemetry.

    - Check things that can panic/can fail:

        - Integer substraction.

        - Slicing/indexing.

        - Functions that can panic.

        - Something else?.

    - Remove help icons, add help menu instead.

    - Improve GUI.

- Someday:

    - The parameters used for filter design are hardcoded, maybe add a toml file
        with constants?

    - Investigate about despeckle.

    - Make OSX binaries, I don't have a Mac. I should cross-compile or get a virtual
        machine to work?.

    - Check OSX build dependencies, now on GNU/Linux we need `libssl-dev`.

    - [Compile as a library and create an Android client](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html)

    - Implement false color [algorithm by enigmastrat](https://github.com/enigmastrat/apt137/tree/feature/false_color)

    - For some reason the `--debug` does not work when using the GUI. Maybe
      replace argparse for something else.

    - Improve syncing performance. Make it faster and more resilent to noise,
        maybe working with the mean and variance?.

    - Try different `WORK_RATE`s.

    - Looks like the Windows version has some icons missing.

    - Live decoding, from a TCP stream or using
        [librtlsdr](https://github.com/steve-m/librtlsdr/blob/master/include/rtl-sdr.h)
        or from audio.

    - Add man page.


## Compilation

Build with `--release`, Rust does some optimizations and it works faster.

### GNU/Linux

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0).

- `sudo apt install libgtk-3-dev libssl-dev`.

- `cargo build --release`.

### GNU/Linux portable

I can't make `gtk-rs` to work with the `x86_64-unknown-linux-musl` target, so
I'm building with the default `x86_64-unknown-linux-gnu` on Debian Jessie. I
think the binary works on any linux with GLIBC newer than the one used when
building, that's why I'm using a Debian Jessie docker image.

So in the end, I build by releases with a Docker image: The GUI version, the no
GUI version and the GUI .deb package.

- Set up:

  - Install Docker.

  - Move to root folder on this repository.

  - `docker build ./build/linux-gnu-docker/ -t noaa-apt-linux-build-image`

  - `docker create -v "$(pwd)":/home/rustacean/src --name noaa-apt-linux-build noaa-apt-linux-build-image`

- Building the binaries:

  - `docker start -ai noaa-apt-linux-build`

- The binaries/packages are on `./target/docker_builds`

### Mac / OSX

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0). The 'unix installer' is fine for Macs.

- Install dependencies via [Homebrew](https://brew.sh/). I'm not entirely sure
  if these are enough:
  `brew install gtk+3 adwaita-icon-theme openssl`.

- `cargo build --release`.

### Windows portable

I never tried to compile from Windows, I cross-compile from GNU/Linux to
Windows. I tried to get a mingw64-gtk environment to work on Debian without
success. So I use a modification of a Docker image I found
[here](https://github.com/LeoTindall/rust-mingw64-gtk-docker).

- Set up:

  - Install Docker.

  - Move to root folder on this repository.

  - `docker build ./build/windows-gnu-docker/ -t noaa-apt-windows-build-image`

  - `docker create -v $(pwd):/home/rustacean/src --name noaa-apt-windows-build noaa-apt-windows-build-image`.

- Building the package:

  - `docker start -ai noaa-apt-windows-build`.

- The binaries/packages are on `./target/docker_builds`

### Raspberry Pi

I'm building it using the same docker container I use for GNU/Linux portables.

I followed [this guide](https://hackernoon.com/seamlessly-cross-compiling-rust-for-raspberry-pis-ede5e2bd3fe2).
Modified the Docker image based on
[this container](https://github.com/sdt/docker-raspberry-pi-cross-compiler).

## Tests

Unit tests are located on the bottom of every module.

```
cargo test
```

Also, for GNU/Linux I have a bash script that runs the program on WAV files
located on `/test/`. Results are on `/test/results/`, check with Audacity.

```
./test/test.sh
```

## Release checklist

- Update dependencies: `cargo update`.

- Unit tests: `cargo test`.

- Get previous tags from remote and check latest version, just in case:

    ```
    git fetch --tag
    git tag -l
    ```

- Increment version number on `/Cargo.toml`.

- Increment version number on `/docs/version_check`.

- Increment version number on `/src/program.rc`.

- Write changelog on `/debian/changelog`.

- Edit the Downloads page on the website, point to the new packages that are
    going to be uploaded.

- Build using Docker for:

    - GNU/Linux.

    - GNU/Linux without GUI.

    - Windows.

- [Check required glibc version](https://www.agardner.me/golang/cgo/c/dependencies/glibc/kernel/linux/2015/12/12/c-dependencies.html),
    should be less than the version shown on the Download page.

- Check archives, names should be:

    - `noaa-apt-?.?.?-x86_64-linux-gnu.zip`

    - `noaa-apt-?.?.?-x86_64-linux-gnu-nogui.zip`

    - `noaa-apt_?.?.?-1_amd64.deb`

    - `noaa-apt-?.?.?-x86_64-windows-gnu.zip`

- Test both GNU/Linux builds using `/test/test.sh`.

- Test Windows version.

- Optionally test `.deb` on Ubuntu VM.

- Create tag on git, e.g.: `git tag v0.9.2`.

- Push tag, e.g.: `git push origin v0.9.2`.

- Edit release on GitHub. Leave "Release title" empty, leave changelog as
    description. Upload files.

## Check for updates

The program sends a HTTP GET request and receives the latest version available,
the URL is ``https://noaa-apt.mbernardi.com.ar/version_check?{current version}``.

The currently installed version is sent just in case I want to track which
versions are being used by people. Anyways, for now I delegated the ``noaa-apt``
to Github and ``version_check`` is just a static file being served using Github
pages, so currently I'm not logging any information. In the future I won't log
anything other than the currently installed version.

## Misc

- When I tried to UDP stream from GQRX to `localhost` it didn't work, I had to
    change the address to `127.0.0.1`.

- [Sizes for Windows icons](https://docs.microsoft.com/en-us/windows/desktop/uxguide/vis-icons#size-requirements):
    16x16, 32x32, 48x48, and 256x256. Icons generated using the script
    `/build/generate_windows_icon.sh`.

- Decode of `argentina.wav` on Raspberry Pi took approx 36s with WXtoImg
    (pristine, no map, no despeckle) and 1:30s with noaa-apt. Something like 3X
    slower.

## Acknowledgements

- RTL-SDR.com: For writing a blog post.

- pietern: I took the AM demodulator from his [apt137 decoder][apt137].

- Grant T. Olson: OSX build instructions.

- wren84: Reported problems with input noise filtering.

- FMighty: Helped with cross compilation to Raspberry Pi.

## References

- [Error Handling in Rust][1].

- [Python GTK+ 3 Tutorial][2]: For Python but I like the Widget Gallery.

- [Cross-compiling from Ubuntu to Windows with Rustup][3].

- [How to compile C GTK3+ program in Ubuntu for windows?][4].

- [rust-mingw64-gtk Docker image][5]: I took the Windows Dockerfile from there.

[1]: https://blog.burntsushi.net/rust-error-handling/
[2]: https://python-gtk-3-tutorial.readthedocs.io/en/latest/index.html
[3]: https://www.reddit.com/r/rust/comments/5k8uab/crosscompiling_from_ubuntu_to_windows_with_rustup/
[4]: https://askubuntu.com/questions/942010/how-to-compile-c-gtk3-program-in-ubuntu-for-windows
[5]: https://github.com/LeoTindall/rust-mingw64-gtk-docker
[apt137]: https://github.com/pietern/apt137
