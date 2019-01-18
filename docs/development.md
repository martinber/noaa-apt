---
title: Development
layout: main
---

---

**Contents**

- TOC
{:toc}

---

The documentation is everything on this website. This website is built from the
`/docs` folder on the repository. Also there are a lot of comments on the code
because I tend to forget everything quite fast.
[The repository is available on GitHub](https://github.com/martinber/noaa-apt).

## Things to do

- Image contrast from telemetry bands.

- The parameters used for filter design are hardcoded, maybe add a toml file
  with constants?

- Make OSX binaries, I don't have a Mac. I should cross-compile or get a virtual
  machine to work?.

- Check OSX build dependencies, now on GNU/Linux we need `libssl-dev`.

- For some reason the `--debug` does not work when using the GUI.

- Improve syncing performance. Improve hardcoded sync frame.

## Compilation

**Build with `--release`, Rust does some optimizations and it works MUCH
faster. Really, otherwise it takes FOREVER.**

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

- Set up:

  - Install Docker.

  - Move to root folder on this repository.

  - `docker build ./build/linux-gnu-docker/ -t noaa-apt-linux-build-image`

  - `docker create -v "$(pwd)":/home/rustacean/src --name noaa-apt-linux-build noaa-apt-linux-build-image`

- Building the binaries:

  - `docker start -ai noaa-apt-linux-build`

- The binaries are on `./target/x86_64-unknown-linux-gnu/package` and on
    `./target/x86_64-unknown-linux-gnu/package`.

### Mac / OSX

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0). The 'unix installer' is fine for Macs.

- Install dependencies via [Homebrew](https://brew.sh/). I'm not entirely sure
  if these are enough:
  `brew install gtk+3 adwaita-icon-theme openssl`. TODO

- `cargo build --release`.

### Windows portable

I never tried to compile from Windows, I cross-compile from GNU/Linux to
Windows. I tried to get a mingw64-gtk environment to work on Debian without
success. So I use a Docker image I found
[here](https://github.com/LeoTindall/rust-mingw64-gtk-docker).

- Set up:

  - Install Docker.

  - Move to root folder on this repository.

  - `docker build ./build/windows-gnu-docker/ -t noaa-apt-windows-build-image`

  - `docker create -v $(pwd):/home/rustacean/src --name noaa-apt-windows-build noaa-apt-windows-build-image`.

- Building the package:

  - `docker start -ai noaa-apt-windows-build`.

  - The build is on `./target/x86_64-pc-windows-gnu/package/`.

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

- Increment version number on `/Cargo.lock`.

- Increment version number on `/docs/version_check`.

- Build using Docker for:

    - GNU/Linux.

    - GNU/Linux without GUI.

    - Windows.

- Compress with the `/test` folder. Delete `/test/test.sh` on the Windows
  archive. Names:

    - `noaa-apt-?.?.?-x86_64-linux-gnu.zip`

    - `noaa-apt-?.?.?-x86_64-linux-gnu-nogui.zip`

    - `noaa-apt-?.?.?-x86_64-windows-gnu.zip`

- Extract somewhere both GNU/Linux builds and test using `/test/test.sh`.

- Test Windows version.

- Create tag on git, e.g.: `git tag v0.9.2`.

- Push tag, e.g.: `git tag v0.9.2`.

- Edit release on GitHub. Leave "Release title" empty, check commits and leave
    changelog as description. Upload zip files.

- Edit the Downloads page on the website.

## Website

I'm using Jekyll, the website is built automatically by GitHub from the `/docs`
folder. These are the steps to build the website locally:

```
sudo apt-get install ruby-dev
gem install bundler jekyll
cd docs # Important!
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

## Misc

- When I tried to UDP stream from GQRX to `localhost` it didn't work, I had to
    change the address to `127.0.0.1`.

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
