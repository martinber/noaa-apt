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
  --document-private-items --open`

- Also there are a lot of comments on the code because I tend to forget
  how it works quite fast.

## Code style

- We use rustfmt, run with `cargo fmt`.

- Should follow the [Style guidelines] but 80 characters as line width.

- Docstrings: Try to document everything, follow [RFC-1574] without using links
  or examples.

- Order of `use` in code (generally also alphabetically):

    ```
    pub mod ...;
    mod ...;
    pub use std::...;
    pub use thirdparty::...;
    pub use crate::...;
    use std::...;
    use thirdparty::...;
    use crate::...;
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

- To set the sizes of youtube videos
    [I use this](https://css-tricks.com/NetMag/FluidWidthVideo/Article-FluidWidthVideo.php).

## Things to do

- Important:

    - Look for defaults for yaw and scales?. See if yaw problems are caused by
        az_ref?. Ask someone.

    - Allow different timezones per filename.

    - The program eats RAM when opening bad custom TLE files.

    - More information link not working on Windows.

    - Add latitude longitude grid.

- Someday:

    - Option to select a pre set path to save images (and notify DG4EK)

    - Implement histogram equalization on floats, before converting pixel values
        to integers.

    - Show user position on map. Add support for
        [gpsd](https://gpsd.gitlab.io/gpsd/index.html). Maybe using
        [gpsd_proto](https://crates.io/crates/gpsd_proto).

    - Draw image over mercator (or at least equirectangular) projection.

    - Make seconds spinner add a minute when wrapping.

    - Check things that can panic/can fail:

        - Integer substraction.

        - Slicing/indexing.

        - Functions that can panic.

        - Something else?.

        - Make sure that Rate cant overflow when resampling against strange
            sample rates.

    - The error system is a mess. I'd like to make it consistent. Functions
        should add context to errors using for example `map_err()`?.

    - Use new Rust `async` syntax where needed?.

    - Show telemetry bands on GUI.

    - Log everything to a file, especially for Windows since it doesn't have a
      console to see output.

    - Investigate about despeckle.

    - Make OSX binaries, I don't have a Mac. I should cross-compile or get a virtual
        machine to work?.

    - Check OSX build dependencies, now on GNU/Linux we need `libssl-dev`.

    - [Compile as a library and create an Android client](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html)

    - Improve syncing performance. Make it faster and more resilent to noise,
        maybe working with the mean and variance?. Especially for Raspberry Pi.

    - Live decoding, from a TCP stream or using
        [librtlsdr](https://github.com/steve-m/librtlsdr/blob/master/include/rtl-sdr.h)
        or from audio.

    - Add man page.

    - Show panics from the decoding thread on GUI. Looks like I have to wait
      until rust creates a simple way, the current alternatives are:

        - `std::thread::JoinHandle::join()` returns `Err()` on a panic, but
            blocks the GUI thread.

        - `std::panic::catch_unwind` works "only" for unwinding panics, maybe
            it's enough?

        - `#[panic_handler]` looks useful for `no_std` applications.

## Compilation

Build with `--release`, Rust does some optimizations and it works faster.

### GNU/Linux

- Install [rustup](https://rustup.rs/)

- `sudo apt install libgtk-3-dev libssl-dev`.

If building with GUI:

- `sudo apt install libgtk-3-dev`

- `cargo build --release`.

If building without GUI:

- `cargo build --release --no-default-features`.

It is important to be located in the root folder of this repository. Run with:

    ./target/release/noaa-apt

If you are moving the executable somewhere, it is important to move along the
`res/` folder, because it is needed in the working directory during runtime. For
example:

    mkdir ~/Desktop/noaa-apt/
    cp ./target/release/noaa-apt ~/Desktop/noaa-apt/
    cp -r ./res ~/Desktop/noaa-apt/
    cp ./build/run-noaa-apt.sh ~/Desktop/noaa-apt/

### GNU/Linux portable

I can't make `gtk-rs` to work with the `x86_64-unknown-linux-musl` target, so
I'm building with the default `x86_64-unknown-linux-gnu` on Debian Stretch. I
think the binary works on any linux with GLIBC newer than the one used when
building, that's why I'm using a Debian Jessie docker image.

So in the end, I build by releases with a Docker image: The GUI version, the no
GUI version and the GUI .deb package. Also I build the Raspberry Pi versions
(armhf).

- Set up:

  - Install Docker.

  - Move to root folder on this repository.

  - `docker build ./build/linux-gnu-docker/ -t noaa-apt-linux-build-image`

  - `docker create -v "$(pwd)":/home/rustacean/src --name noaa-apt-linux-build noaa-apt-linux-build-image`

- Building the binaries:

  - `docker start -ai noaa-apt-linux-build`

- The binaries/packages are on `./target/docker_builds`

### Mac / OSX

You can use the
[Nix Package](https://search.nixos.org/packages?query=noaa-apt&from=0&size=30&sort=relevance&channel=unstable),
but I don't know how this works. If someone knows please write a guide.

The only method I know is by using Homebrew. This is a summary of
[this extensive installation guide](https://publiclab.org/notes/sashae/07-02-2020/how-to-compile-noaa-apt-1-2-0-on-mac).

- Install [rustup](https://rustup.rs/):

      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      source $HOME/.cargo/env

- Install dependencies via [Homebrew](https://brew.sh/).

      /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
      brew install gtk+3 adwaita-icon-theme openssl

- Download the [source code zip file](https://github.com/martinber/noaa-apt/releases)
  and extract it somewhere.

- Move to the downloaded folder

      cd Downloads/noaa-apt-1.2.0/

- Compile

      export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig/
      cargo build --release

There has been also a [succesful installation by installing first in OSX
Catalina and then copying the folder to OSX
Sierra](https://github.com/martinber/noaa-apt/issues/38). Apparently it is
necessary to have Homebrew and have the packages updated, including `gtk+3`,
`adwait-icon-theme`, `openssl` and `harfbuzz`.

Run with:

    ./target/release/noaa-apt

### Android with Termux

You will need a good device and quite a lot of patience. The instructions to
compile are the same as in GNU/Linux. It may be necesary to set the
`OPENSSL_DIR` variable to point to the correct path.

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

## Tests

Unit tests are located on the bottom of every module.

```
cargo test
```

There is an additional test that needs a [N2YO.com](https://www.n2yo.com) API
key.

```
N2YO_KEY=ASJKDH... cargo test -- --ignored
```

Also, for GNU/Linux I have a bash script that runs the program on WAV files
located on `/test/`. Results are on `/test/results/`, check with Audacity.

```
./test/test.sh
```

I use Clippy too:

- `cargo clippy -- -A clippy::ptr_arg -A clippy::match_bool -A clippy::manual_range_contains -A clippy::module_inception`: Should have no warnings.

- `cargo clippy -- -A clippy::ptr_arg -W clippy::pedantic`: Check once in a
    while but ignore most of the lints

## Release checklist

- Update dependency version constraints on `Cargo.toml`:
    `cargo install cargo-edit; cargo upgrade`.

- Update dependencies (on `Cargo.lock`): `cargo update`.

- Unit tests: `cargo test`.

- Get previous tags from remote and check latest version, just in case:

    ```
    git fetch --tag
    git tag -l
    ```

- Increment version number on `/Cargo.toml`.

- Increment version number on `/docs/version_check`.

- Increment version number on `/src/program.rc`.

- Increment version number of `/src/default_settings.toml` if necessary.

- Increment version number of settings file in `/src/config.rs` if necessary.

- Copy `/src/default_settings.toml` to `/docs/default_settings.toml` if
    necessary.

- Write changelog on `/debian/changelog`.

- Edit the Downloads page on the website, point to the new packages that are
    going to be uploaded. Also change the version in the install command.

- Build using Docker for:

    - GNU/Linux.

    - GNU/Linux without GUI.

    - Windows.

- Check required glibc version, should be less than the version shown on the
    Download page. Use `/build/check_glibc.sh`, e.g.:
    `./build/check_glibc.sh ./target/docker_builds/noaa-apt-?.?.?-x86_64-linux-gnu/noaa-apt`

- Check archives, names should be:

    - `noaa-apt-?.?.?-x86_64-linux-gnu.zip`

    - `noaa-apt-?.?.?-x86_64-linux-gnu-nogui.zip`

    - `noaa-apt-?.?.?-armv7-linux-gnueabihf.zip`

    - `noaa-apt-?.?.?-armv7-linux-gnueabihf-nogui.zip`

    - `noaa-apt_?.?.?-1_amd64.deb`

    - `noaa-apt-?.?.?-x86_64-windows-gnu.zip`

- Test both GNU/Linux builds using `/test/test.sh`.

- Test Windows version.

- Test Raspberry Pi version.

- Optionally test `.deb` on Ubuntu VM.

- Create tag on git, e.g.: `git tag v0.9.2`.

- Push tag, e.g.: `git push origin v0.9.2`.

- Edit release on GitHub. Leave "Release title" empty, leave changelog as
    description. Upload files.

- Ask Sylogista to update the [AUR package](https://aur.archlinux.org/packages/noaa-apt/).

- Ask Tom Repetti to update the [NixOS Package](https://github.com/NixOS/nixpkgs/pull/96519).

## Check for updates

The program sends a HTTP GET request and receives the latest version available,
the URL is ``https://noaa-apt.mbernardi.com.ar/version_check?{current version}``.

The currently installed version is sent just in case I want to track which
versions are being used by people. Anyways, for now I delegated the ``noaa-apt``
to Github and ``version_check`` is just a static file being served using Github
pages, so currently I'm not logging any information. In the future I won't log
anything other than the currently installed version.

If you want to disable this you can do it from the
[configuration file](./usage.html#configuration-file).

## Misc

- When I tried to UDP stream from GQRX to `localhost` it didn't work, I had to
    change the address to `127.0.0.1`.

- [Sizes for Windows icons](https://docs.microsoft.com/en-us/windows/desktop/uxguide/vis-icons#size-requirements):
    16x16, 32x32, 48x48, and 256x256. Icons generated using the script
    `/build/generate_windows_icon.sh`.

- Decode of `argentina.wav` on Raspberry Pi took approx 36s with WXtoImg
    (pristine, no map, no despeckle) and 46s with noaa-apt using the fast
    profile.

## Thank you to

- RTL-SDR.com: For writing
    [a blog post](https://www.rtl-sdr.com/new-noaa-apt-image-decoder/).

- Arcadie Z.: Histogram equalization, image rotation, false color, fixes and
    more.

- pietern: I took the AM demodulator from his [apt137 decoder][apt137].

- Ossi Herrala: Did some fixes and updated dependencies.

- Nodraak: Added tips when saving files.

- Dirk Lison: Testing and filename guessing for WXtoIMG.

- Grant T. Olson: Early OSX build instructions

- Sasha Engelmann and Bill Liles: [Further OSX build instructions](https://publiclab.org/notes/sashae/07-02-2020/how-to-compile-noaa-apt-1-2-0-on-mac).

- FMighty: Helped with cross compilation to Raspberry Pi.

- Sylogista: Maintains an [AUR package](https://aur.archlinux.org/packages/noaa-apt/).

- Tom Repetti: [Packaged](https://github.com/NixOS/nixpkgs/pull/96519) program for NixOS.

- ButchOfBurdock: Instructions for compilation in Android + Termux.

- Peter Vogel: For writing about noaa-apt on the web.

- wren84 and Florentin314: Reported problems with decoded images.

- Gagootron, xxretartistxx and unknownantipatriot: Provided example images of
    missing samples.

- BGNLouie, nosduh2, samarrangepas, KiwiEntropy, budude2, K2SDR, K6KZO,
    TheLAX18: Provided recordings to test map overlays.

- JKamue, Kind-Shame: Provided example images for website.

- To everyone who reported issues or sent me emails showing their images!

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
