---
title: Download
layout: main
---

## Download

You can download executables for Linux or Windows from the
[releases page](https://github.com/martinber/noaa-apt/releases). Your options
are:

- Linux:

  - Last version binary: Has GUI. Needs GTK and GLIBC version at least 2.19. I
    think that should work in most common distros.

  - Build yourself the last version.

  - Version 0.9.1 binary: Doesn't have GUI, only terminal. Should work
    everywhere.

- Windows:

  - Download binary for the last version.

  - Build yourself the last version (never tried to do that from Windows).

- OSX:

  - Build yourself the last version.

## Compiling

### Linux

**Build with `--release`, Rust does some optimizations and it works MUCH
faster. Really, otherwise it takes FOREVER.**

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0).

- `sudo apt install libgtk-3-dev`.

- `cargo build --release`.

### Linux portable

I can't make `gtk-rs` to work with the `x86_64-unknown-linux-musl` target, so
I'm building with the default `x86_64-unknown-linux-gnu` on Debian Jessie. I
think the binary works on any linux with GLIBC newer than the one used when
building, that's why I'm using a Debian Jessie docker image.

- Set up:

  - Install Docker.

  - `sudo apt install libgtk-3-dev`.

  - Move to root folder.

  - `docker build ./linux-docker/ -t noaa-apt-linux-build-image`.

  - `docker create -v $(pwd):/src --name noaa-apt-linux-build noaa-apt-linux-build-image`.

- Building the binary:

  - `docker start -ai noaa-apt-linux-build`.

  - The build is on `./target/x86_64-unknown-linux-gnu/`.

### Mac / OSX

**Build with `--release`, Rust does some optimizations and it works MUCH
faster. Really, otherwise it takes FOREVER.**

- Install [rustup](https://rustup.rs/) (you need `rustc --version` at least
  1.27.0). The 'unix installer' is fine for Macs.

- Install dependencies via [Homebrew](https://brew.sh/):
  `brew install gtk+3 adwaita-icon-theme`.

- `cargo build --release`.

### Windows portable

I never tried to compile from Windows, I cross-compile from Linux to Windows. I
tried to get a mingw64-gtk environment to work on Debian without success. So I
use a Docker image I found
[here](https://github.com/LeoTindall/rust-mingw64-gtk-docker).

- Set up:

  - Install Docker.

  - `sudo apt install libgtk-3-dev`.

  - Move to root folder.

  - `docker build ./windows-docker/ -t noaa-apt-windows-build-image`.

  - `docker create -v $(pwd):/home/rustacean/src --name noaa-apt-windows-build noaa-apt-windows-build-image`.

- Building the package:

  - `docker start -ai noaa-apt-windows-build`.

  - The build is on `./target/x86_64-pc-windows-gnu/package/`.
